// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

//! The ClientHandler responsible of nostr clients and subscriptions.

use bitcoin::secp256k1;
use bitcoin::secp256k1::SecretKey;
use bitcoin::secp256k1::Secp256k1;

use nostr::{RelayMessage, Event, ClientMessage, SubscriptionId, Filter};
use nostr::key::XOnlyPublicKey;

use crate::config::Config;

use crate::{events, NostrSub, NostrClient};
use crate::events::{ClientEvents, EventsProvider, ServerCmd};
use crate::nostr_db::DbRequest;
use crate::util::is_ephemeral;

use futures_util::{future, pin_mut, TryStreamExt, StreamExt, SinkExt};

use tokio::net::TcpStream;
use tokio::sync::Mutex;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::{thread, time};

use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::{sleep, Duration};

use tokio_tungstenite::tungstenite::Message;

/// Max number of subscriptions by connected clients.
const MAX_SUBSCRIPTIONS: u64 = 100;

//pub(crate) struct NostrSub {
//	our_side_id: u64,
//	id: SubscriptionId,
//	filters: Vec<Filter>,
//}


const MAGIC_SERVER_PAYLOAD: [u8; 4] = [0x27, 0x27, 0x27, 0x27];

//TODO: rework the mutex model
pub struct ClientHandler {
	clients: HashMap<u64, NostrClient>,
	subscriptions: HashMap<u64, NostrSub>,

	clients_counter: u64,
	subscriptions_counter: u64,

	map_send: Mutex<HashMap<u64, mpsc::UnboundedSender<Vec<u8>>>>,
	map_receive: Mutex<HashMap<u64, mpsc::UnboundedReceiver<Vec<u8>>>>,

	handler_receive: Mutex<mpsc::UnboundedReceiver<ClientEvents>>,
	connection_receive: Mutex<mpsc::UnboundedReceiver<(TcpStream, SocketAddr)>>,

	send_db_requests: Mutex<mpsc::UnboundedSender<DbRequest>>,
	handler_receive_db_result: Mutex<mpsc::UnboundedReceiver<ClientEvents>>,

	filtered_events: HashMap<SubscriptionId, Event>,

	pending_events: Mutex<Vec<ClientEvents>>,

	config: Config
}

async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr, outgoing_receive: mpsc::UnboundedSender<Vec<u8>>, mut incoming_send: mpsc::UnboundedReceiver<Vec<u8>>) {
	println!("[CIVKITD] - NET: incoming tcp Connection from :{}", addr);

	let mut ws_stream = tokio_tungstenite::accept_async(raw_stream).await.expect("Error during the websocket handshake occured");
	println!("[CIVKITD] - NET: websocket established: {}", addr);

	let (mut outgoing, mut incoming) = ws_stream.split();

	tokio::spawn(async move {
		while let Some(message) = incoming.next().await {
			match message {
				Ok(Message::Text(msg)) => { outgoing_receive.send(msg.into()); },
				Ok(Message::Binary(msg)) => { outgoing_receive.send(msg); },
				Ok(Message::Close(None)) => { break; },
				_ => {
					//TODO: if failure client state cleanly
					panic!("[CIVKITD] - NOSTR: unknown webSocket message ?!"); 
				},
			}
		}
		println!("[CIVKITD] websocket connection closing: {}", addr);
		//TODO: if closing clean client and thread state
	});

	tokio::spawn(async move {
		while let Some(message) = incoming_send.recv().await {
			if message == MAGIC_SERVER_PAYLOAD {
				match outgoing.close().await {
					Ok(_) => {},
					Err(_) => { println!("[CIVKITD] - NOSTR: sample disconnect !"); },
				}
			} else {
				match outgoing.send(Message::Binary(message)).await {
					Ok(_) => {},
					Err(_) => { println!("[CIVKITD] - NOSTR: error sample sending !"); },
				}
			}
		}
	});
}

impl ClientHandler {
	pub fn new(handler_receive: mpsc::UnboundedReceiver<ClientEvents>, connection_receive: mpsc::UnboundedReceiver<(TcpStream, SocketAddr)>, send_db_requests: mpsc::UnboundedSender<DbRequest>, handler_receive_db_result: mpsc::UnboundedReceiver<ClientEvents>, our_config: Config) -> Self {

		let (outgoing_receive, incoming_receive) = mpsc::unbounded_channel::<Vec<u8>>();

		ClientHandler {
			clients: HashMap::new(),
			subscriptions: HashMap::new(),

			clients_counter: 0,
			subscriptions_counter: 0,

			map_send: Mutex::new(HashMap::new()),
			map_receive: Mutex::new(HashMap::new()),

			handler_receive: Mutex::new(handler_receive),
			connection_receive: Mutex::new(connection_receive),

			send_db_requests: Mutex::new(send_db_requests),
			handler_receive_db_result: Mutex::new(handler_receive_db_result),

			filtered_events: HashMap::new(),

			pending_events: Mutex::new(vec![]),

			config: our_config
		}
	}

	pub async fn run(&mut self) {
		loop {
			sleep(Duration::from_millis(1000)).await;

			let mut client_events = Vec::new();
			{
				// We receive an offer processed by the relay management utility, or any other
				// service-side Nostr event.
				let mut handler_receive_lock = self.handler_receive.lock();

				if let Ok(event) = handler_receive_lock.await.try_recv() {
					println!("[CIVKITD] - PROCESSING: received an event from service manager");
					// Handle server requests
					if let ClientEvents::Server{ cmd } = event {
						match cmd {
							ServerCmd::GetClients { respond_to } => {
								let all_clients = self.clients.values().cloned().collect::<Vec<NostrClient>>();
								let _ = respond_to.send(all_clients);
							},
							ServerCmd::DisconnectClient { client_id } => {
								let map_send_lock = self.map_send.lock();
								if let Some(outgoing_send) = map_send_lock.await.get(&client_id) {
									match outgoing_send.send(MAGIC_SERVER_PAYLOAD.clone().to_vec()) {
										Ok(_) => {},
										Err(_) => { println!("[CIVKITD] - NOSTR: Error inter thread sending disconnect"); }
									}
								}
							},
						}
					} else {
						client_events.push(event);
					}
				}
			}

			{
				// We receive a result of a db query from the DB manager.
				let mut handler_receive_db_result_lock = self.handler_receive_db_result.lock();
				if let Ok(event) = handler_receive_db_result_lock.await.try_recv() {
					client_events.push(event);
				}
			}

			for event in client_events {
				let mut map_send_lock = self.map_send.lock();

				for (id, outgoing_send) in map_send_lock.await.iter() {
					println!("[CIVKITD] - NOSTR: sending event for client {}", id);
					match event {
						ClientEvents::TextNote { ref event } => {
							let random_id = SubscriptionId::generate();
							let relay_message = RelayMessage::new_event(random_id, event.clone());
							let serialized_message = relay_message.as_json();
							match outgoing_send.send(serialized_message.into_bytes()) {
								Ok(_) => {},
								Err(_) => { println!("[CIVKITD] - NOSTR: Error inter thread sending note"); }
							}
						},
						ClientEvents::RelayNotice { ref message } => {
							//TODO: implement `requestcredential` announcement
							let relay_message = RelayMessage::new_notice(message);
							let serialized_message = relay_message.as_json();
							match outgoing_send.send(serialized_message.into_bytes()) {
								Ok(_) => {},
								Err(_) => { println!("[CIVKITD] - NOSTR: Error inter thread sending notice"); },
							}
						},
						ClientEvents::OrderNote { ref order } => {
							let random_id = SubscriptionId::generate();
							let relay_message = RelayMessage::new_event(random_id, order.clone());
							let serialized_message = relay_message.as_json();
							match outgoing_send.send(serialized_message.into_bytes()) {
								Ok(_) => {},
								Err(_) => { println!("[CIVKITD] - NOSTR: Error inter thread sending order"); }
							}
						},
						ClientEvents::StoredEvent { ref client_id,  ref events } => {
							if id != client_id { continue }

							//TODO: capture the subscription id
							let random_id = SubscriptionId::generate();
							for ev in events {
								let relay_message = RelayMessage::new_event(random_id.clone(), ev.clone());
								let serialized_message = relay_message.as_json();
								match outgoing_send.send(serialized_message.into_bytes()) {
									Ok(_) => {},
									Err(_) => { println!("[CIVKITD] - NOSTR: Error inter thread sending stored events"); },
								}
							}
							let relay_message = RelayMessage::new_eose(random_id.clone());
							let serialized_message = relay_message.as_json();
							match outgoing_send.send(serialized_message.into_bytes()) {
								Ok(_) => {},
								Err(_) => { println!("[CIVKITD] - NOSTR: Error inter thread sending end of stored events"); },
							}
						},
						_ => {}
					}
				}
			}

			let mut dispatch_events = Vec::new();
			{

				let mut pending_events_lock = self.pending_events.lock().await;
				dispatch_events.append(&mut pending_events_lock);
			}

			// Dispatch pending client events
			{
				let mut map_send_lock = self.map_send.lock();
				for (map_client_id, outgoing_send) in map_send_lock.await.iter() {
					for event in dispatch_events.iter() {
						match event {
							ClientEvents::EndOfStoredEvents { client_id, sub_id } => {
								if client_id == map_client_id {
									let relay_message = RelayMessage::new_eose(sub_id.clone());
									let serialized_message = relay_message.as_json();
									match outgoing_send.send(serialized_message.into_bytes()) {
										Ok(_) => {},
										Err(_) => { println!("[CIVKITD] - NOSTR: Error inter thread sending end of stored events"); },
									}
								}
							},
							ClientEvents::SubscribedEvent { client_id, sub_id, event } => {
								if client_id == map_client_id {
									let relay_message = RelayMessage::new_event(sub_id.clone(), event.clone());
									let serialized_message = relay_message.as_json();
									match outgoing_send.send(serialized_message.into_bytes()) {
										Ok(_) => {},
										Err(_) => { println!("[CIVKITD] - NOSTR: Error inter thread sending subcribed event"); },
									}
								}
							},
							_ => {},
						}
					}
				}
			}

			let mut socket_and_sender = None;
			{
				// We receive a new Nostr client connection.
				let mut nostr_client_request_lock = self.connection_receive.lock();

				if let Ok((stream, addr)) = nostr_client_request_lock.await.try_recv() {
					let (outgoing_send, incoming_send) = mpsc::unbounded_channel::<Vec<u8>>();
					let (outgoing_receive, incoming_receive) = mpsc::unbounded_channel::<Vec<u8>>();
					socket_and_sender = Some((addr.clone(), outgoing_send, incoming_receive));
					tokio::spawn(async move {
						handle_connection(stream, addr, outgoing_receive, incoming_send).await;
					});
				}
			}

			let mut write_db = Vec::new();

			if let Some((addr, outgoing_send, incoming_receive)) = socket_and_sender {
				self.clients_counter += 1;
				let client_id = self.clients_counter;
				let new_nostr_client = NostrClient::new(client_id as u64, addr);
				let client_2 = new_nostr_client.clone();
				self.clients.insert(client_id, new_nostr_client);
				{
					let mut map_send_lock = self.map_send.lock();
					map_send_lock.await.insert(client_id, outgoing_send);
				}
				{
					let mut map_receive_lock = self.map_receive.lock();
					map_receive_lock.await.insert(client_id, incoming_receive);
				}
				let db_request = DbRequest::WriteClient(client_2);
				write_db.push(db_request);
			}

			let mut msg_queue = Vec::new();
			{
				// We check if a Nostr client has sent a new event
				let mut map_receive_lock = self.map_receive.lock();
				for (id, mut incoming_receive) in map_receive_lock.await.iter_mut() {
					if let Ok(msg) = incoming_receive.try_recv() {
						msg_queue.push((id.clone(), msg.clone()));
					}
				}
			}

			let mut new_pending_events = Vec::new();
			{
				// If we have a new event, we'll fan out according to its types (event, subscription, close)
				for (id, msg) in msg_queue {
					let msg_json = String::from_utf8(msg).unwrap();
					println!("[CIVKITD] - NOSTR: Message received from {}!", id);
					if let Ok(client_msg) = ClientMessage::from_json(msg_json) {
						match client_msg {
							ClientMessage::Event(msg) => {
								if let Some(nostr_client) = self.clients.get_mut(&id) {
									if !nostr_client.has_pubkey() {
										nostr_client.add_pubkey(msg.pubkey.clone());
									}
								}
								let msg_2 = msg.clone();
								self.filter_events(*msg).await;
								//TODO: we should link our filtering policy to our db storing,
								//otherwise this is a severe DoS vector
								if !is_ephemeral(&msg_2) {
									let db_request = DbRequest::WriteEvent(*msg_2);
									write_db.push(db_request);
								}
							},
							ClientMessage::Req { subscription_id, filters } => {
								self.subscriptions_counter += 1;
								let our_side_id = self.subscriptions_counter;
								// Check this client number of subscriptions
								if let Some(nostr_client) = self.clients.get_mut(&id) {
									//TODO: NIP 01 : "Clients should not open more than one websocket to each relay. One channel can support an unlimited number of subscriptions, so clients should do that."
									// Sanitize with keys ?
									if !nostr_client.add_sub(our_side_id, MAX_SUBSCRIPTIONS) {
										println!("[CIVKITD] - NOSTR: subscription register failure");
									}
								}
								let nostr_sub = NostrSub::new(our_side_id, subscription_id.clone(), filters.clone());
								let nostr_sub2 = nostr_sub.clone();
								self.subscriptions.insert(our_side_id, nostr_sub);
								let db_request = DbRequest::ReplayEvents { client_id: id, filters: filters };
								write_db.push(db_request);
							},
							ClientMessage::Close(subscription_id) => {
								//TODO: replace our_side_id by Sha256 of SubscriptionId
								let mut our_side_id = 0;
								for (registered_id, nostr_sub) in self.subscriptions.iter() {
									if nostr_sub.is_our_id(&subscription_id) {
										our_side_id = *registered_id;
									}
								}
								if our_side_id != 0 {
									self.subscriptions.remove(&our_side_id);
									println!("[CIVKITD] - NOSTR: Remove subscription id {}", our_side_id);
								}
							},
							_ => { println!("[CIVKITD] - NOSTR: Unknown client message"); }
						}
					} else { println!("[CIVKITD] - NOSTR: ClientMessage deserialization failure"); }
				}
			}

			{
				for ev in write_db {
					let mut send_db_requests_lock = self.send_db_requests.lock();
					send_db_requests_lock.await.send(ev);
				}
			}


			{
				let mut pending_events_lock = self.pending_events.lock();
				pending_events_lock.await.append(&mut new_pending_events);
			}
		}
	}

	async fn filter_events(&mut self, event: Event) -> bool {

		let mut match_result = false;
		for (our_side_id, sub) in self.subscriptions.iter() {
			let filters = sub.get_filters();
			for filter in filters {
				if let Some(ref kinds) = filter.kinds {
					for kind in kinds.iter() {
						if kind == &event.kind {
							match_result = true;
						}
					}
				}
			}
			let mut clients_to_dispatch = Vec::new();
			if match_result {
				for (client_id, nostr_client) in self.clients.iter() {
					if nostr_client.has_sub(*our_side_id) {
						//TODO: fulfill with match subscription
						let associated_event = ClientEvents::SubscribedEvent { client_id: client_id.clone(), sub_id: SubscriptionId::generate(), event: event.clone() };
						clients_to_dispatch.push(associated_event);
					}
				}
			}

			{
				let mut pending_events_lock = self.pending_events.lock();
				pending_events_lock.await.append(&mut clients_to_dispatch);
			}
		}

		match_result
	}
}
