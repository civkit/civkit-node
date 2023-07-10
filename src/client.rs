// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use boardctrl::board_ctrl_client::BoardCtrlClient;
use boardctrl::{PingRequest, PongRequest, ShutdownRequest, ShutdownReply, SendNote, ReceivedNote, ListClientRequest, ListSubscriptionRequest, PeerConnectionRequest, DisconnectClientRequest, SendNotice};

use std::env;
use std::process;

use clap::{Parser, Subcommand};

pub mod boardctrl {
	tonic::include_proto!("boardctrl");
}

#[derive(Parser, Debug)]
struct Cli {
	/// The port of the connected server
	#[clap(short, long, default_value = "50031")]
	port: String,

	#[clap(subcommand)]
	command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
	/// Send a ping message
	Ping,
	/// Shutdown the connected CivKit node
	Shutdown,
	/// Send a demo NIP-01 EVENT kind 1 to all the connected clients
	Publishtextnote,
	/// List information about connected clients
	Listclients,
	/// List information about subscriptions [TODO]
	Listsubscriptions,
	/// Connect to a BOLT8 peer on local port
	Connectpeer {
		/// The port number for the peer
		peer_local_port: String,
	},
	/// Disconnect from a client [TODO]
	Disconnectclient,
	/// Send a demo NIP-01 NOTICE to all the connected clients
	Publishnotice,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

	let cli = Cli::parse();

	let mut client = BoardCtrlClient::connect(format!("http://[::1]:{}", cli.port)).await?;

	match cli.command {
		Command::Ping => {
			let request = tonic::Request::new(PingRequest {
				name: "PING".into(),
			});

			let response = client.ping_handle(request).await?;

			println!("[CIVKIT-CLI] {}", response.into_inner().name);
		}
		Command::Shutdown => {
			let request = tonic::Request::new(ShutdownRequest {});

			let response = client.shutdown_handle(request).await?;
		}
		Command::Publishtextnote => {
			let request = tonic::Request::new(SendNote {
				content: String::from("Hello World !"),
			});

			let response = client.publish_text_note(request).await?;

			println!("[CIVKIT-CLI] {}", response.into_inner().name);
		}
		Command::Listclients => {
			let request = tonic::Request::new(ListClientRequest {});

			let response = client.list_clients(request).await?;

			println!("[CIVKIT-CLI] clients {:#?}", response.into_inner().clients);
		}
		Command::Listsubscriptions => {
			let request = tonic::Request::new(ListSubscriptionRequest {});

			let response = client.list_subscriptions(request).await?;

			println!("[CIVKIT-CLI] subscriptions {}", response.into_inner().subscriptions);
		}
		Command::Connectpeer { peer_local_port } => {
			let request = tonic::Request::new(PeerConnectionRequest {
				local_port: u64::from_str_radix(&peer_local_port, 10).unwrap()
			});
			let response = client.connect_peer(request).await?;
		}
		Command::Disconnectclient => {
			//TODO: take real client id from input
			let request = tonic::Request::new(DisconnectClientRequest {
				client_id: 0,
			});

			let _response = client.disconnect_client(request).await?;
		}
		Command::Publishnotice => {
			let request = tonic::Request::new(SendNotice {
				info_message: String::from("This is a notice"),
			});

			let response = client.publish_notice(request).await?;
		}
	}
	Ok(())
}
