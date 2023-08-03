// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

/// Main server of the CivKit Node, orchestrate all the components.

mod boardmanager;
mod config;



use crate::config::Config as LocalConfig;

use std::fs;
use crate::boardmanager::ServiceManager;
use civkit::nostr_db::DbRequest;
use civkit::config::Config;
use civkit::clienthandler::{NostrClient, ClientHandler};
use civkit::anchormanager::AnchorManager;
use civkit::credentialgateway::CredentialGateway;
use civkit::kindprocessor::NoteProcessor;
use civkit::nodesigner::NodeSigner;
use civkit::peerhandler::{NoiseGateway, PeerInfo};

use civkit::oniongateway::OnionBox;

use civkit::events::{ClientEvents, EventsProvider, ServerCmd};

use lightning::offers::offer::Offer;

use lightning_invoice::Invoice;

use boardctrl::board_ctrl_server::{BoardCtrl, BoardCtrlServer};

use clap::Parser;

use nostr::{Keys, EventBuilder};

use std::env;
use std::net::SocketAddr;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;


use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio::sync::{oneshot, mpsc};

use tokio_tungstenite::WebSocketStream;

use tonic::{transport::Server, Request, Response, Status};
mod util;
use util::get_default_data_dir;

//TODO: rename boarctrl to something like relayctrl ?
pub mod boardctrl {
	tonic::include_proto!("boardctrl");
}

#[tonic::async_trait]
impl BoardCtrl for ServiceManager {
	async fn ping_handle(&self, request: Request<boardctrl::PingRequest>) -> Result<Response<boardctrl::PongRequest>, Status> {
		let pong = boardctrl::PongRequest {
			name: format!("{}", request.into_inner().name).into(),
		};

		Ok(Response::new(pong))
	}

	async fn shutdown_handle(&self, request: Request<boardctrl::ShutdownRequest>) -> Result<Response<boardctrl::ShutdownReply>, Status> {
		println!("[CIVKITD] - CONTROL: CivKit node shuting down...");
		process::exit(0x0);
	}

	async fn publish_text_note(&self, request: Request<boardctrl::SendNote>) -> Result<Response<boardctrl::ReceivedNote>, Status> {
		let note_content = request.into_inner().content;

		let service_keys = Keys::generate();

		if let Ok(kind1_event) = EventBuilder::new_text_note(note_content, &[]).to_event(&service_keys) {

			let mut service_send_lock = self.service_events_send.lock().unwrap();
			service_send_lock.send(ClientEvents::TextNote { event: kind1_event });
		}

		let received_note = boardctrl::ReceivedNote {
			name: format!("Note publication scheduled").into(),
		};

		Ok(Response::new(received_note))
	}

	async fn disconnect_client(&self, request: Request<boardctrl::DisconnectClientRequest>) -> Result<Response<boardctrl::DisconnectClientReply>, Status> {
		let disconnect_request = request.into_inner().client_id;

		{
			let mut service_send_lock = self.service_events_send.lock().unwrap();
			service_send_lock.send(ClientEvents::Server { cmd: ServerCmd::DisconnectClient { client_id: disconnect_request }});
		}

		Ok(Response::new(boardctrl::DisconnectClientReply {}))
	}

	async fn connect_peer(&self, request: Request<boardctrl::PeerConnectionRequest>) -> Result<Response<boardctrl::PeerConnectionReply>, Status> {
		let peer_port = request.into_inner().local_port;

		println!("[CIVKITD] - CONTROL: sending port to noise gateway !");
		if peer_port > 0 {
			let mut board_peers_lock = self.service_peers_send.lock().unwrap();

			let peer_info = PeerInfo::new(peer_port);
			board_peers_lock.send(peer_info);
		}

		Ok(Response::new(boardctrl::PeerConnectionReply {}))
	}

	async fn list_peers(&self, request: Request<boardctrl::ListPeersRequest>) -> Result<Response<boardctrl::ListPeersReply>, Status> {

		let peers_query = boardctrl::ListPeersReply {
			peers: 1,
		};

		Ok(Response::new(peers_query))
	}

	async fn list_clients(&self, request: Request<boardctrl::ListClientRequest>) -> Result<Response<boardctrl::ListClientReply>, Status> {
		println!("[CIVKITD] - CONTROL: sending list-clients request to ClientHandler!");
		let (send, recv) = oneshot::channel::<Vec<NostrClient>>();
		{
			let mut board_send_lock = self.service_events_send.lock().unwrap();
			board_send_lock.send(ClientEvents::Server { cmd: ServerCmd::GetClients { respond_to: send }});
		}
		let response = recv.await.expect("ClientHandler has been killed");
		
		let board_clients: Vec<boardctrl::Client> = response
    		.iter()
    		.map(|client| {
				boardctrl::Client {
					pubkey: client.pubkey.map(|s| s.to_string()).unwrap_or("".to_string()),
					client_id: client.client_id,
					associated_socket: client.associated_socket.to_string(),
					subscriptions: client.subscriptions.len() as u64,
				}
			})
			.collect();
		let client_query = boardctrl::ListClientReply {
			clients: board_clients,
		};
	
		Ok(Response::new(client_query))
	}

	async fn list_subscriptions(&self, request: Request<boardctrl::ListSubscriptionRequest>) -> Result<Response<boardctrl::ListSubscriptionReply>, Status> {

		let sub_query = boardctrl::ListSubscriptionReply {
			subscriptions: 1,
		};

		Ok(Response::new(sub_query))
	}

	async fn status_handle(&self, request: Request<boardctrl::BoardStatusRequest>) -> Result<Response<boardctrl::BoardStatusReply>, Status> {

		//TODO give a mspc communication channel between ServiceManager and NoteProcessor
		let notes = 0;
		//let notes = self.note_stats();

		let board_status = boardctrl::BoardStatusReply {
			offers: notes,
		};

		Ok(Response::new(board_status))
	}

	async fn publish_notice(&self, request: Request<boardctrl::SendNotice>) -> Result<Response<boardctrl::ReceivedNotice>, Status> {
		let notice_message = request.into_inner().info_message;

		let service_keys = Keys::generate();

		{
			let mut board_send_lock = self.service_events_send.lock().unwrap();
			board_send_lock.send(ClientEvents::RelayNotice { message: notice_message });
		}

		let received_note = boardctrl::ReceivedNote {
			name: format!("Note publication scheduled").into(),
		};

		Ok(Response::new(boardctrl::ReceivedNotice {}))
	}

	async fn publish_offer(&self, request: Request<boardctrl::SendOffer>) -> Result<Response<boardctrl::ReceivedOffer>, Status> {
		let offer_message = request.into_inner().offer;

		let service_keys = Keys::generate();

		if let Ok(offer) = Offer::try_from(offer_message) {
			let encoded_offer = offer.to_string();
			if let Ok(kind32500_event) = EventBuilder::new_order_note(encoded_offer, &[]).to_event(&service_keys)
			{
				let mut board_send_lock = self.service_events_send.lock().unwrap();
				board_send_lock.send(ClientEvents::OrderNote { order: kind32500_event });
			}
		}

		Ok(Response::new(boardctrl::ReceivedOffer {}))
	}

	async fn publish_invoice(&self, request: Request<boardctrl::SendInvoice>) -> Result<Response<boardctrl::ReceivedInvoice>, Status> {
		let invoice_message = request.into_inner().invoice;

		let service_keys = Keys::generate();
		//let invoice: Invoice = serde_json::from_str(&invoice_message).unwrap();
		//let encoded_invoice = invoice.to_string();
		if let Ok(kind32500_event) = EventBuilder::new_order_note(invoice_message, &[]).to_event(&service_keys)
		{
				let mut board_send_lock = self.service_events_send.lock().unwrap();
				board_send_lock.send(ClientEvents::OrderNote { order: kind32500_event });
		}

		Ok(Response::new(boardctrl::ReceivedInvoice {}))
	}

	async fn list_db_events(&self, request: Request<boardctrl::ListDbEventsRequest>) -> Result<Response<boardctrl::ListDbEventsReply>, Status> {

		println!("[CIVKITD] - CONTROL: listing DB event !");

		{
			let mut send_db_request_lock = self.send_db_request.lock().unwrap();
			send_db_request_lock.send(DbRequest::DumpEvents);
		}

		Ok(Response::new(boardctrl::ListDbEventsReply {}))
	}
}

#[derive(Parser, Debug)]
struct Cli {
	/// The port to listen for BOLT8 peers
	#[clap(long, short = 'p', default_value = "9735")]
	noise_port: String,
	/// Nostr relay port
	#[clap(short, long, default_value = "50021")]
	nostr_port: String,
	/// The port to listen for CLI connections
	#[clap(short, long, default_value = "50031")]
	cli_port: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the default data directory
    let data_dir = get_default_data_dir();
    println!("Log file path: {:?}", data_dir.join("debug.log"));

    // Create the necessary directories
    fs::create_dir_all(&data_dir)?;

    // Attempt to read the contents of the config file
    let contents = fs::read_to_string("../../config.toml");
    match contents {
        Ok(contents) => {
            // Open a log file for writing
            let mut log_file = fs::File::create(data_dir.join("debug.log"))?;

            // Attempt to deserialize the config file
            let config: Result<LocalConfig, _> = toml::from_str(&contents);
            match config {
                Ok(config) => {
                    println!("{:#?}", config);
                    // Write the parsed configuration to the log file
                    writeln!(log_file, "{:#?}", config)?;
                },
                Err(err) => {
                    // Log the error to the file
                    writeln!(log_file, "Could not deserialize the config file: {:?}", err)?;
                    // Handle the error
                    return Err(err.into());
                }
            }
        },
        Err(err) => {
            // Log the error to the file
            let mut log_file = fs::File::create(data_dir.join("debug.log"))?;
            writeln!(log_file, "Something went wrong reading the file: {:?}", err)?;
            // Handle the error
            return Err(err.into());
        }
    }

	let cli = Cli::parse();
	println!("[CIVKITD] - INIT: CivKit node starting up...");
	//TODO add a Logger interface

	println!("[CIVKITD] - INIT: noise port {} nostr port {} cli_port {}", cli.noise_port, cli.nostr_port, cli.cli_port);

	let rt = Runtime::new()?;

	// We initialize the communication channels between the service manager and ClientHandler.
	let (board_events_send, handler_receive) = mpsc::unbounded_channel::<ClientEvents>();

	// We initialize the communication channels between the service manager and NoiseGateway.
	let (board_peer_send, gateway_receive) = mpsc::unbounded_channel::<PeerInfo>();

	// We initialize the communication channels between the nostr tcp listener and ClientHandler.
	let (socket_connector, request_receive) = mpsc::unbounded_channel::<(TcpStream, SocketAddr)>();

	// We initialize the communication channels between the NoteProcessor and ClientHandler.
	let (handler_send_dbrequests, processor_receive_dbrequests) = mpsc::unbounded_channel::<(DbRequest)>();

	// We initialize the communication channels between the NoteProcessor and ServiceManager.
	let (manager_send_dbrequests, receive_dbrequests_manager) = mpsc::unbounded_channel::<(DbRequest)>();

	// The onion message handler...quite empty for now.
	let onion_box = OnionBox::new();

	// The noise peers handler...almost empty for now.
	let noise_gateway = NoiseGateway::new(gateway_receive);

	// The staking credentials handler...quite empty for now.
	let credential_gateway = Arc::new(CredentialGateway::new());

	// The note or service provider...quite empty for now.
	let mut note_processor = NoteProcessor::new(processor_receive_dbrequests, receive_dbrequests_manager);

	// The service provider signer...quite empty for now.
	let node_signer = Arc::new(NodeSigner::new());

	// The chain notirazation handler...quite empty for now.
	let anchor_manager = Arc::new(AnchorManager::new());

	// Main handler of Nostr connections.
	let mut client_handler = ClientHandler::new(handler_receive, request_receive, handler_send_dbrequests, config.clone());

	// Main handler of services provision.
	let board_manager = ServiceManager::new(credential_gateway, node_signer, anchor_manager, board_events_send, board_peer_send, manager_send_dbrequests, config.clone());

	let addr = format!("[::1]:{}", cli.cli_port).parse()?;

	let board_svc = Server::builder()
		.add_service(BoardCtrlServer::new(board_manager))
		.serve(addr);

	let peer_manager = noise_gateway.peer_manager.clone();
	let stop_listen_connect = Arc::new(AtomicBool::new(false));
	let stop_listen = Arc::clone(&stop_listen_connect);

	rt.block_on(async {

	// We start the gRPC server for `civkit-cli`.
    	tokio::spawn(async move {
		if let Err(e) = board_svc.await {
			eprintln!("Error = {:?}", e);
		}
	});

	// We start the NIP-01 relay for clients.
	tokio::spawn(async move {
		client_handler.run().await;
	});

	// We start the onion box for received onions.
	tokio::spawn(async move {
		onion_box.run().await;
	});

	// We start the note processor for messages.
	tokio::spawn(async move {
		note_processor.run().await;
	});

	// We start the noise gateway for BOLT8 peers.
	tokio::spawn(async move {
		noise_gateway.run().await;
	});

	// We start the tcp listener for BOLT8 peers.
	tokio::spawn(async move {
		let listener = tokio::net::TcpListener::bind(format!("[::1]:{}", cli.noise_port)).await.expect("Failed to bind to listen port");

		loop {
			let inbound_peer_mgr = peer_manager.clone();
			let tcp_stream = listener.accept().await.unwrap().0;
			println!("[CIVKITD] - NET: inbound noise connection !");
			if stop_listen.load(Ordering::Acquire) {
				return;
			}
			tokio::spawn(async move {
				lightning_net_tokio::setup_inbound(
					inbound_peer_mgr,
					tcp_stream.into_std().unwrap(),
				)
				.await;
			});
		}
	});

	// We start the tcp listener for NIP-01 clients.
	tokio::spawn(async move {
		let try_socket = TcpListener::bind(format!("[::1]:{}", cli.nostr_port)).await;
		let listener = try_socket.expect("Failed to bind");

		println!("[CIVKITD] - NET: ready to listen tcp connection for clients !");
		while let Ok((stream, addr)) = listener.accept().await {
			println!("[CIVKITD] - NET: receive a tcp connection !");
			socket_connector.send((stream, addr));
		}
	});

	loop {}

	});

    	Ok(())
}

