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

pub mod boardctrl {
	tonic::include_proto!("boardctrl");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

	let command = env::args().nth(1).unwrap_or_else(|| {
		println!("[CIVKIT-CLI] command required!");
		process::exit(0x100);
	});

	//TODO: do proper unix input
	let cli_port_n = if command == "connectpeer" {
		let peer_local_port = env::args().nth(2).unwrap_or_else(|| {
			println!("[CIVKIT-CLI] connectpeer peer_local_port requireed!");
			process::exit(0x100);
		});
		3
	} else { 2 };

	let cli_port = env::args().nth(cli_port_n).unwrap_or_else(|| "50031".to_string());

	let mut client = BoardCtrlClient::connect(format!("http://[::1]:{}", cli_port)).await?;

	match command.as_str() {
		"ping" => {
			let request = tonic::Request::new(PingRequest {
				name: "PING".into(),
			});

			let response = client.ping_handle(request).await?;

			println!("[CIVKIT-CLI] {}", response.into_inner().name);
		},
		"shutdown" => {
			let request = tonic::Request::new(ShutdownRequest {});

			let response = client.shutdown_handle(request).await?;
		},
		"publishtextnote" => {
			let request = tonic::Request::new(SendNote {
				content: String::from("Hello World !"),
			});

			let response = client.publish_text_note(request).await?;

			println!("[CIVKIT-CLI] {}", response.into_inner().name);
		},
		"listclient" => {
			let request = tonic::Request::new(ListClientRequest {});

			let response = client.list_clients(request).await?;

			println!("[CIVKIT-CLI] clients {}", response.into_inner().clients);
		},
		"listsubscriptions" => {
			let request = tonic::Request::new(ListSubscriptionRequest {});

			let response = client.list_subscriptions(request).await?;

			println!("[CIVKIT-CLI] subscriptions {}", response.into_inner().subscriptions);
		},
		"connectpeer" => {
			let peer_local_port = env::args().nth(2).unwrap();
			let request = tonic::Request::new(PeerConnectionRequest {
				local_port: u64::from_str_radix(&peer_local_port, 10).unwrap()
			});

			let response = client.connect_peer(request).await?;
		},
		"disconnectclient" => {
			//TODO: take real client id from input
			let request = tonic::Request::new(DisconnectClientRequest {
				client_id: 0,
			});

			let _response = client.disconnect_client(request).await?;
		},
		"publishnotice" => {
			let request = tonic::Request::new(SendNotice {
				info_message: String::from("This is a notice"),
			});

			let response = client.publish_notice(request).await?;
		},
		_ => {
			println!("[CIVKIT-CLI] unknown command");
		},
	}

	Ok(())
}
