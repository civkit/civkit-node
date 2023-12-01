// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

//! An interface to sanitize and enforce service policy on the received notes.


use nostr::Filter;

use crate::mainstay::send_commitment;

use crate::events::ClientEvents;
use crate::nostr_db::DbRequest;
use crate::nostr_db::{write_new_subscription_db, write_new_event_db, write_new_client_db, print_events_db, print_clients_db, query_events_db, get_cumulative_hash_of_last_event};

use nostr::Event;

use crate::config::Config;

use std::sync::Mutex;

use crate::util::is_replaceable;

use tokio::sync::mpsc;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration};
use base64::encode;

use std::collections::HashMap;

const MAX_PENDING_DB_REQUEST_PER_CLIENT: u64 = 100;

pub struct NoteProcessor {
	note_counters: Mutex<u64>,
	current_height: u64,

	receive_db_requests: TokioMutex<mpsc::UnboundedReceiver<DbRequest>>,
	send_db_result_handler: TokioMutex<mpsc::UnboundedSender<ClientEvents>>,

	receive_db_requests_manager: TokioMutex<mpsc::UnboundedReceiver<DbRequest>>,
	receive_validation_dbrequests_manager: TokioMutex<mpsc::UnboundedReceiver<ClientEvents>>,

	pending_write_db: HashMap<u64, Vec<(u64, Event)>>,

	config: Config,
}

impl NoteProcessor {
	pub fn new(receive_db_requests: mpsc::UnboundedReceiver<DbRequest>, receive_db_requests_manager: mpsc::UnboundedReceiver<DbRequest>, send_db_result_handler: mpsc::UnboundedSender<ClientEvents>, receive_validation_dbrequests_manager: mpsc::UnboundedReceiver<ClientEvents>, our_config: Config) -> Self {
		NoteProcessor {
			note_counters: Mutex::new(0),
			current_height: 0,

			receive_db_requests: TokioMutex::new(receive_db_requests),
			send_db_result_handler: TokioMutex::new(send_db_result_handler),

			receive_db_requests_manager: TokioMutex::new(receive_db_requests_manager),
			receive_validation_dbrequests_manager: TokioMutex::new(receive_validation_dbrequests_manager),

			pending_write_db: HashMap::new(),

			config: our_config,
		}
	}

	pub fn process_note(&self, note: Vec<u8>) -> bool {
		println!("Received a note !");

		if let Ok(mut note_counters_lock) = self.note_counters.lock() {
			(*note_counters_lock) += 1;
		}

		return true;
	}

	pub fn note_stats(&self) -> u64 {
		let mut notes = 0;
		if let Ok(note_counters_lock) = self.note_counters.lock() {
			notes = *note_counters_lock;
		}
		return notes;
	}

	pub async fn run(&mut self) {
		loop {
			sleep(Duration::from_millis(1000)).await;

			//TODO: wait for the ServiceDeliveranceResult of the CredentialGateway before to trigger write_new_event_db.

			let mut replay_request = Vec::new();
			let mut ok_events = Vec::new();
			{
				let mut receive_db_requests_lock = self.receive_db_requests.lock();
				if let Ok(db_request) = receive_db_requests_lock.await.try_recv() {
					match db_request {
						DbRequest::WriteEvent { client_id, deliverance_id, ev } => { self.pending_write_db.insert(client_id, vec![(deliverance_id, ev)]); },
						DbRequest::WriteSub(ns) => { write_new_subscription_db(ns); },
						DbRequest::WriteClient(ct) => { write_new_client_db(ct).await; },
						DbRequest::ReplayEvents { client_id, filters } => { replay_request.push((client_id, filters)); },
						_ => {},
					}
					println!("[CIVKITD] - NOTE PROCESSING: Note processor received DB requests");
				}
			}

			let mut paid_and_validated_events = Vec::new();
			{
				let mut receive_validation_dbrequests_manager_lock = self.receive_validation_dbrequests_manager.lock();
				if let Ok(paid_and_validated_event) = receive_validation_dbrequests_manager_lock.await.try_recv() {
					paid_and_validated_events.push(paid_and_validated_event);
				}
			}

			for client_ev in paid_and_validated_events {
				match client_ev {
					ClientEvents::ValidationResult { client_id, deliverance_id, event } => {
						if let Some(queue_events) = self.pending_write_db.get(&client_id) {
							for queue_event in queue_events {
								if queue_event.0 == deliverance_id {
									let event_id = event.id;
									if is_replaceable(&event) {
										//TODO: build filter and replace event
										//TODO: If two events have the same timestamp, the event with the lowest id SHOULD be retained, and the other discarded
										let filter = Filter::new();
										if let Ok(old_ev) = query_events_db(filter) {
											//TODO: check if you should query for multiple replaced events
											write_new_event_db(event.clone(), Some(old_ev)).await;
										}
									} else {
										let ret = write_new_event_db(event.clone(), None).await;
										if ret { ok_events.push(event_id); }
									}
								}
							}
						}
					},
					_ => {},
				}
			}

			{
				let mut receive_db_requests_manager_lock = self.receive_db_requests_manager.lock();
				if let Ok(db_request) = receive_db_requests_manager_lock.await.try_recv() {
					match db_request {
						DbRequest::DumpEvents => { print_events_db().await; },
						DbRequest::DumpClients => { print_clients_db().await; },
						_ => {},
					}
					println!("[CIVKITD] - NOTE PROCESSING: Note processor received DB requests from ServiceManager");
				}
			}

			let mut event_replay_result = Vec::new();
			for req in replay_request {
				let mut client_id_result = Vec::with_capacity(0);
				for filter in req.1 {
					if let Ok(events) = query_events_db(filter) {
						client_id_result = events;
					}
				}
				event_replay_result.push((req.0.clone(), client_id_result));
			}

			for ret in event_replay_result {
				let mut send_db_result_handler_lock = self.send_db_result_handler.lock();
				let stored_event = ClientEvents::StoredEvent { client_id: ret.0, events: ret.1 };
				send_db_result_handler_lock.await.send(stored_event);
			}

			for ev in ok_events {
				let mut send_db_result_handler_lock = self.send_db_result_handler.lock();
				let ok_event = ClientEvents::OkEvent { event_id: ev, ret: true, msg: None };
				let event_id = ev.to_string();
				let commitment = encode(get_cumulative_hash_of_last_event().await.unwrap());
				let position = self.config.mainstay.position;
				let token = &self.config.mainstay.token;
											
				let req = send_commitment(commitment.as_str(), position as u64, token, &self.config.mainstay).await.unwrap();

				match req.send().await {
					Ok(_) => println!("Commitment sent successfully"),
					Err(err) => println!("Error sending commitment: {}", err),
				}
			}
		}
	}
}
