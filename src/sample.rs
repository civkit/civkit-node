// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use std::env;
use std::io;
use std::io::Write;
use std::process;

use bitcoin::secp256k1::{PublicKey, SecretKey, Secp256k1};

use nostr::{RelayMessage, EventBuilder, Metadata, Keys, ClientMessage, Kind, Filter, SubscriptionId, Timestamp};

use url::Url;

use futures_channel;
use futures_util::{future, pin_mut, StreamExt, TryStreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, tungstenite::error::Error};

use std::str::FromStr;

async fn poll_for_user_input(client_keys: Keys, tx: futures_channel::mpsc::UnboundedSender<Message>) {

	println!("Civkit sample startup successful. Enter \"help\" to view available commands");

	loop {
		print!("> ");
		io::stdout().flush().unwrap();
		let mut line = String::new();
		if let Err(e) = io::stdin().read_line(&mut line) {
			break println!("ERROR {}", e);
		}

		if line.len() == 0 {
			break;
		}

		let mut words = line.split_whitespace();
		if let Some(word) = words.next() {
			match word {
				//TODO: implement help command
				"help" => { println!("help command - to be implemented"); },
				"sendtextnote" => {
					let content = words.next();
					if content.is_none() {
						println!("ERROR: sendtextnote has 1 required argument: `sendtextnote content`");
						continue;
					}
					if let Ok(kind1_event) = EventBuilder::new_text_note(content.unwrap(), &[]).to_event(&client_keys) {
						let client_message = ClientMessage::new_event(kind1_event);
						let serialized_message = client_message.as_json();
						tx.unbounded_send(Message::text(serialized_message)).unwrap();
					}
				},
				"setmetadata" => {
					let username = words.next();
					let about = words.next();
					let picture = words.next();
					if username.is_none() || about.is_none() || picture.is_none() {
						println!("ERROR: setmetadata has 3 required arguments: `setmetadata username about picture");
					}
					//TODO: add picture arg
					let metadata = Metadata::new().name(username.unwrap()).about(about.unwrap());
					if let Ok(kind0_event) = EventBuilder::set_metadata(metadata).to_event(&client_keys) {
						let client_message = ClientMessage::new_event(kind0_event);
						let serialized_message = client_message.as_json();
						tx.unbounded_send(Message::text(serialized_message)).unwrap();
					}
				},
				"recommendserver" => {
					let urlrelay = words.next();
					if urlrelay.is_none() {
						println!("ERROR: recommendserver has 1 required argument: `recommendserver urlrelay");
					}
					if let Ok(kind2_event) = EventBuilder::add_recommended_relay(&Url::parse(urlrelay.unwrap()).unwrap()).to_event(&client_keys) {
						let client_message = ClientMessage::new_event(kind2_event);
						let serialized_message = client_message.as_json();
						tx.unbounded_send(Message::text(serialized_message)).unwrap();
					}
				},
				"opensubscription" => {
					let subscriptionid = words.next();
					let kinds_raw = words.next();
					let since_raw = words.next();
					let until_raw = words.next();
					if subscriptionid.is_none() || kinds_raw.is_none() || since_raw.is_none() || until_raw.is_none() {
						println!("ERROR: opensubscription has 5 required arguments: `opensubscription subscriptionid kinds since until");
					}
					let id = SubscriptionId::new(subscriptionid.unwrap());
					let kinds_vec: Vec<&str> = kinds_raw.unwrap().split(',').collect();
					let mut kinds = Vec::with_capacity(kinds_vec.len());
					for kind in kinds_vec {
						if let Ok(k) = Kind::from_str(kind) {
							kinds.push(k);
						}
					}
					let since = Timestamp::from_str(since_raw.unwrap()).unwrap();
					let until = Timestamp::from_str(until_raw.unwrap()).unwrap();
					let filter = Filter::new().kinds(kinds).since(since).until(until);
					let client_message = ClientMessage::new_req(id, vec![filter]);
					let serialized_message = client_message.as_json();
					tx.unbounded_send(Message::text(serialized_message)).unwrap();
				},
				"closesubscription" => {
					let subscriptionid = words.next();
					if subscriptionid.is_none() {
						println!("ERROR: closesubscription has 1 required argument: `closesubscription subscriptionid");
					}
					let id = SubscriptionId::new(subscriptionid.unwrap());
					let client_message = ClientMessage::close(id);
					let serialized_message = client_message.as_json();
					tx.unbounded_send(Message::text(serialized_message)).unwrap();
				},
				"shutdown" => {
					tx.unbounded_send(Message::Close(None)).unwrap();
					tx.close_channel();
					println!("Civkit sample exiting...");
					process::exit(0x0100);
				},
				_ => { println!("Unknown command !"); },
			}
		}
	}
}

async fn poll_for_server_output(mut rx: futures_channel::mpsc::UnboundedReceiver<Message>) {

	loop {
		if let Ok(message) = rx.try_next() {
			let msg = message.unwrap();
			let msg_json = String::from_utf8(msg.into()).unwrap();
			//println!("Received message {}", msg_json);
			if let Ok(relay_msg) = RelayMessage::from_json(msg_json) {
				match relay_msg {
					RelayMessage::Event { subscription_id, event } => {
						//TODO: NIP 01: `EVENT` messages MUST be sent only with a subscriptionID related to a subscription previously initiated by the client (using the `REQ` message above)`
						println!("\n[EVENT] {}", event.content);
						print!("> ");
						io::stdout().flush().unwrap();
					},
					RelayMessage::Notice { message } => {
						println!("\n[NOTICE] {}", message);
						print!("> ");
						io::stdout().flush().unwrap();
					},
					RelayMessage::EndOfStoredEvents(sub_id) => {
						println!("\n[EOSE] {}", sub_id);
						print!("> ");
						io::stdout().flush().unwrap();
					},
					_ => { println!("Unknown server message"); }
				}
			} else { println!("RelayMessage deserialization failure"); }
		}
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

	let connect_addr = env::args().nth(1).unwrap_or_else(|| "50021".to_string());

	let addr = format!("ws://[::1]:50021");

	let url = url::Url::parse(&addr).unwrap();

	// Init client state
	let keys = Keys::generate();

	let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
	tokio::spawn(poll_for_user_input(keys, stdin_tx));

	let (stdout_tx, stdout_rx) = futures_channel::mpsc::unbounded();
	tokio::spawn(poll_for_server_output(stdout_rx));

	let (ws_stream, _) = if let Ok(info) = connect_async(url).await {
		info 
	} else {
		panic!("WebSocket connection failed !");
	};

	let (write, read) = ws_stream.split();

	let stdin_to_ws = stdin_rx.map(Ok).forward(write);
	let ws_to_stdout = read.try_for_each(|msg| {
		stdout_tx.unbounded_send(msg).unwrap();
		future::ok(())
	});

	pin_mut!(stdin_to_ws, ws_to_stdout);
	future::select(stdin_to_ws, ws_to_stdout).await;
	Ok(())
}
