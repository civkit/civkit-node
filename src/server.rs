// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

/// Main server of the CivKit Node, orchestrate all the components.

mod servicemanager;
mod config;
mod util;

use crate::util::init_logger;
use log;
use std::fs;
use crate::servicemanager::ServiceManager;
use civkit::nostr_db::DbRequest;
use civkit::config::Config;
use civkit::clienthandler::ClientHandler;
use civkit::anchormanager::AnchorManager;
use civkit::credentialgateway::CredentialGateway;
use civkit::kindprocessor::NoteProcessor;
use civkit::nodesigner::NodeSigner;
use civkit::peerhandler::{NoiseGateway, PeerInfo};
use civkit::NostrClient;

use civkit::oniongateway::OnionBox;

use civkit::events::{ClientEvents, EventsProvider, ServerCmd};

use lightning::offers::offer::Offer;

use lightning_invoice::Invoice;

use adminctrl::admin_ctrl_server::{AdminCtrl, AdminCtrlServer};

use crate::civkitservice::civkit_service_server::{CivkitService, CivkitServiceServer};

use clap::Parser;

use nostr::{Keys, EventBuilder};

use std::env;
use std::net::SocketAddr;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio::sync::{oneshot, mpsc};

use tokio_tungstenite::WebSocketStream;

use tonic::{transport::Server, Request, Response, Status};

//TODO: rename boarctrl to something like relayctrl ?
pub mod adminctrl {
	tonic::include_proto!("adminctrl");
}

pub mod civkitservice {
	tonic::include_proto!("civkitservice");
}


#[tonic::async_trait]
impl AdminCtrl for ServiceManager {
	async fn ping_handle(&self, request: Request<adminctrl::PingRequest>) -> Result<Response<adminctrl::PongRequest>, Status> {
		let pong = adminctrl::PongRequest {
			name: format!("{}", request.into_inner().name).into(),
		};

		Ok(Response::new(pong))
	}

	async fn shutdown_handle(&self, request: Request<adminctrl::ShutdownRequest>) -> Result<Response<adminctrl::ShutdownReply>, Status> {
		println!("[CIVKITD] - CONTROL: CivKit node shuting down...");
		process::exit(0x0);
	}

	async fn publish_text_note(&self, request: Request<adminctrl::SendNote>) -> Result<Response<adminctrl::ReceivedNote>, Status> {
		let note_content = request.into_inner().content;

		let service_keys = Keys::generate();

		if let Ok(kind1_event) = EventBuilder::new_text_note(note_content, &[]).to_event(&service_keys) {

			let mut service_send_lock = self.service_events_send.lock().unwrap();
			service_send_lock.send(ClientEvents::TextNote { event: kind1_event });
		}

		let received_note = adminctrl::ReceivedNote {
			name: format!("Note publication scheduled").into(),
		};

		Ok(Response::new(received_note))
	}

	async fn disconnect_client(&self, request: Request<adminctrl::DisconnectClientRequest>) -> Result<Response<adminctrl::DisconnectClientReply>, Status> {
		let disconnect_request = request.into_inner().client_id;

		{
			let mut service_send_lock = self.service_events_send.lock().unwrap();
			service_send_lock.send(ClientEvents::Server { cmd: ServerCmd::DisconnectClient { client_id: disconnect_request }});
		}

		Ok(Response::new(adminctrl::DisconnectClientReply {}))
	}

	async fn connect_peer(&self, request: Request<adminctrl::PeerConnectionRequest>) -> Result<Response<adminctrl::PeerConnectionReply>, Status> {
		let peer_port = request.into_inner().local_port;

		println!("[CIVKITD] - CONTROL: sending port to noise gateway !");
		if peer_port > 0 {
			let mut service_mngr_peers_lock = self.service_peers_send.lock().unwrap();

			let peer_info = PeerInfo::new(peer_port);
			service_mngr_peers_lock.send(peer_info);
		}

		Ok(Response::new(adminctrl::PeerConnectionReply {}))
	}

	async fn list_peers(&self, request: Request<adminctrl::ListPeersRequest>) -> Result<Response<adminctrl::ListPeersReply>, Status> {

		let peers_query = adminctrl::ListPeersReply {
			peers: 1,
		};

		Ok(Response::new(peers_query))
	}

	async fn list_clients(&self, request: Request<adminctrl::ListClientRequest>) -> Result<Response<adminctrl::ListClientReply>, Status> {
		println!("[CIVKITD] - CONTROL: sending list-clients request to ClientHandler!");
		let (send, recv) = oneshot::channel::<Vec<NostrClient>>();
		{
			let mut service_mngr_send_lock = self.service_events_send.lock().unwrap();
			service_mngr_send_lock.send(ClientEvents::Server { cmd: ServerCmd::GetClients { respond_to: send }});
		}
		let response = recv.await.expect("ClientHandler has been killed");
		
		let service_mngr_clients: Vec<adminctrl::Client> = response
    		.iter()
    		.map(|client| {
				adminctrl::Client {
					pubkey: client.pubkey.map(|s| s.to_string()).unwrap_or("".to_string()),
					client_id: client.client_id,
					associated_socket: client.associated_socket.to_string(),
					subscriptions: client.subscriptions.len() as u64,
				}
			})
			.collect();
		let client_query = adminctrl::ListClientReply {
			clients: service_mngr_clients,
		};
	
		Ok(Response::new(client_query))
	}

	async fn list_subscriptions(&self, request: Request<adminctrl::ListSubscriptionRequest>) -> Result<Response<adminctrl::ListSubscriptionReply>, Status> {

		let sub_query = adminctrl::ListSubscriptionReply {
			subscriptions: 1,
		};

		Ok(Response::new(sub_query))
	}

	async fn relay_status_handle(&self, request: Request<adminctrl::ServiceMngrStatusRequest>) -> Result<Response<adminctrl::ServiceMngrStatusReply>, Status> {

		//TODO give a mspc communication channel between ServiceManager and NoteProcessor
		let notes = 0;
		//let notes = self.note_stats();

		let service_mngr_status = adminctrl::ServiceMngrStatusReply {
			offers: notes,
		};

		Ok(Response::new(service_mngr_status))
	}

	async fn publish_notice(&self, request: Request<adminctrl::SendNotice>) -> Result<Response<adminctrl::ReceivedNotice>, Status> {
		let notice_message = request.into_inner().info_message;

		let service_keys = Keys::generate();

		{
			let mut service_mngr_send_lock = self.service_events_send.lock().unwrap();
			service_mngr_send_lock.send(ClientEvents::RelayNotice { client_id: 0, message: notice_message });
		}

		let received_note = adminctrl::ReceivedNote {
			name: format!("Note publication scheduled").into(),
		};

		Ok(Response::new(adminctrl::ReceivedNotice {}))
	}

	async fn publish_offer(&self, request: Request<adminctrl::SendOffer>) -> Result<Response<adminctrl::ReceivedOffer>, Status> {
		let offer_message = request.into_inner().offer;

		let service_keys = Keys::generate();

		if let Ok(offer) = Offer::try_from(offer_message) {
			let encoded_offer = offer.to_string();
			if let Ok(kind32500_event) = EventBuilder::new_order_note(encoded_offer, &[]).to_event(&service_keys)
			{
				let mut service_mngr_send_lock = self.service_events_send.lock().unwrap();
				service_mngr_send_lock.send(ClientEvents::OrderNote { order: kind32500_event });
			}
		}

		Ok(Response::new(adminctrl::ReceivedOffer {}))
	}

	async fn publish_invoice(&self, request: Request<adminctrl::SendInvoice>) -> Result<Response<adminctrl::ReceivedInvoice>, Status> {
		let invoice_message = request.into_inner().invoice;

		let service_keys = Keys::generate();
		//let invoice: Invoice = serde_json::from_str(&invoice_message).unwrap();
		//let encoded_invoice = invoice.to_string();
		if let Ok(kind32500_event) = EventBuilder::new_order_note(invoice_message, &[]).to_event(&service_keys)
		{
				let mut service_mngr_send_lock = self.service_events_send.lock().unwrap();
				service_mngr_send_lock.send(ClientEvents::OrderNote { order: kind32500_event });
		}

		Ok(Response::new(adminctrl::ReceivedInvoice {}))
	}

	async fn list_db_events(&self, request: Request<adminctrl::ListDbEventsRequest>) -> Result<Response<adminctrl::ListDbEventsReply>, Status> {

		println!("[CIVKITD] - CONTROL: listing DB event !");

		{
			let mut send_db_request_lock = self.send_db_request.lock().unwrap();
			send_db_request_lock.send(DbRequest::DumpEvents);
		}

		Ok(Response::new(adminctrl::ListDbEventsReply {}))
	}

	async fn list_db_clients(&self, request: Request<adminctrl::ListDbClientsRequest>) -> Result<Response<adminctrl::ListDbClientsReply>, Status> {

		println!("[CIVKITD] - CONTROL: listing DB clients !");

		{
			let mut send_db_request_lock = self.send_db_request.lock().unwrap();
			send_db_request_lock.send(DbRequest::DumpClients);
		}

		Ok(Response::new(adminctrl::ListDbClientsReply {}))
	}
}

struct DummyManager {}

#[tonic::async_trait]
impl CivkitService for DummyManager {
	async fn register_service(&self, request: Request<civkitservice::RegisterRequest>) -> Result<Response<civkitservice::RegisterReply>, Status> {

		println!("Received registration");

		Ok(Response::new(civkitservice::RegisterReply { registration_result: 1 }))
	}

	async fn fetch_service_event(&self, request: Request<civkitservice::FetchRequest>) -> Result<Response<civkitservice::FetchReply>, Status> {

		println!("Received fetch service");

		Ok(Response::new(civkitservice::FetchReply {}))
	}

	async fn submit_service_event(&self, request: Request<civkitservice::SubmitRequest>) -> Result<Response<civkitservice::SubmitReply>, Status> {

		println!("Submit service");

		Ok(Response::new(civkitservice::SubmitReply {}))
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

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let data_dir = util::get_default_data_dir();

	let config_path = data_dir.join("example-config.toml");

    // Read the configuration file
    let contents = fs::read_to_string(&config_path);
    let config = match contents {
        Ok(data) => {
            toml::from_str(&data).expect("Could not deserialize the config file content")
        },
        Err(_) => {
            // If there's an error reading the file, use the default configuration
            Config::default()
        }
    };
    // Initialize the logger with the level from the config
    util::init_logger(&data_dir, &config.logging.level)?;

    log::info!("Logging initialized. Log file located at: {:?}", data_dir.join("debug.log"));

    // Test logging at different levels
    log::error!("This is a test error message");
    log::warn!("This is a test warning message");
    log::info!("This is a test info message");
    log::debug!("This is a test debug message");
    log::trace!("This is a test trace message");
    // Log the parsed configuration data
    //log::info!("Parsed configuration: {:?}", config);	
	let cli = Cli::parse();
	
	println!("[CIVKITD] - INIT: CivKit node starting up...");
	//TODO add a Logger interface

	println!("[CIVKITD] - INIT: noise port {} nostr port {} cli_port {}", cli.noise_port, cli.nostr_port, cli.cli_port);

	let rt = Runtime::new()?;

	// We initialize the communication channels between the service manager and ClientHandler.
	let (service_mngr_events_send, handler_receive) = mpsc::unbounded_channel::<ClientEvents>();

	// We initialize the communication channels between the service manager and NoiseGateway.
	let (service_mngr_peer_send, gateway_receive) = mpsc::unbounded_channel::<PeerInfo>();

	// We initialize the communication channels between the nostr tcp listener and ClientHandler.
	let (socket_connector, request_receive) = mpsc::unbounded_channel::<(TcpStream, SocketAddr)>();

	// We initialize the communication channels between the NoteProcessor and ClientHandler.
	let (handler_send_dbrequests, processor_receive_dbrequests) = mpsc::unbounded_channel::<(DbRequest)>();

	// We initialize the communication channels between the NoteProcessor and ServiceManager.
	let (manager_send_dbrequests, receive_dbrequests_manager) = mpsc::unbounded_channel::<(DbRequest)>();

	let (send_db_result_handler, handler_receive_db_result) = mpsc::unbounded_channel::<ClientEvents>();

	let (send_credential_events_handler, receive_credential_event_gateway) = mpsc::unbounded_channel::<ClientEvents>();

	// The onion message handler...quite empty for now.
	let onion_box = OnionBox::new();

	// The noise peers handler...almost empty for now.
	let noise_gateway = NoiseGateway::new(gateway_receive);

	// The staking credentials handler...quite empty for now.
	let mut credential_gateway = CredentialGateway::new(receive_credential_event_gateway);

	// The note or service provider...quite empty for now.
	let mut note_processor = NoteProcessor::new(processor_receive_dbrequests, receive_dbrequests_manager, send_db_result_handler);

	// The service provider signer...quite empty for now.
	let node_signer = Arc::new(NodeSigner::new());

	// The chain notirazation handler...quite empty for now.
	let anchor_manager = Arc::new(AnchorManager::new());

	// Main handler of Nostr connections.
	let mut client_handler = ClientHandler::new(handler_receive, request_receive, handler_send_dbrequests, handler_receive_db_result, send_credential_events_handler, config.clone());

	// Main handler of services provision.
	let service_manager = ServiceManager::new(node_signer, anchor_manager, service_mngr_events_send, service_mngr_peer_send, manager_send_dbrequests, config.clone());

	let addr = format!("[::1]:{}", cli.cli_port).parse()?;

	let service_mngr_svc = Server::builder()
		.add_service(AdminCtrlServer::new(service_manager))
		.add_service(CivkitServiceServer::new(DummyManager {}))
		.serve(addr);

	let peer_manager = noise_gateway.peer_manager.clone();
	let stop_listen_connect = Arc::new(AtomicBool::new(false));
	let stop_listen = Arc::clone(&stop_listen_connect);

	rt.block_on(async {

	// We start the gRPC server for `civkit-cli`.
    	tokio::spawn(async move {
		if let Err(e) = service_mngr_svc.await {
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

	// We start the credentials gateway
	// TODO: give a channel with ClientHandler
	tokio::spawn(async move {
		credential_gateway.run().await;
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

