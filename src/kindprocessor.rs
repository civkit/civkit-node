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

use crate::events::ClientEvents;
use crate::nostr_db::DbRequest;
use crate::nostr_db::{write_new_subscription_db, write_new_event_db, write_new_client_db, print_events_db, print_clients_db, query_events_db};

use std::sync::Mutex;

use crate::util::is_replaceable;

use tokio::sync::mpsc;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration};

use crate::mainstay::{send_commitment};
use crate::config::Config;

pub struct NoteProcessor {
	note_counters: Mutex<u64>,
	current_height: u64,

	receive_db_requests: TokioMutex<mpsc::UnboundedReceiver<DbRequest>>,
	send_db_result_handler: TokioMutex<mpsc::UnboundedSender<ClientEvents>>,

	receive_db_requests_manager: TokioMutex<mpsc::UnboundedReceiver<DbRequest>>,
	config: Config
}

impl NoteProcessor {
	pub fn new(receive_db_requests: mpsc::UnboundedReceiver<DbRequest>, receive_db_requests_manager: mpsc::UnboundedReceiver<DbRequest>, send_db_result_handler: mpsc::UnboundedSender<ClientEvents>, our_config: Config) -> Self {
		NoteProcessor {
			note_counters: Mutex::new(0),
			current_height: 0,

			receive_db_requests: TokioMutex::new(receive_db_requests),
			send_db_result_handler: TokioMutex::new(send_db_result_handler),

			receive_db_requests_manager: TokioMutex::new(receive_db_requests_manager),
			config: our_config
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

			let mut replay_request = Vec::new();
			let mut ok_events = Vec::new();
			{
				let mut receive_db_requests_lock = self.receive_db_requests.lock();
				if let Ok(db_request) = receive_db_requests_lock.await.try_recv() {
					match db_request {
						DbRequest::WriteEvent(ev) => {
							let event_id = ev.id;
							if is_replaceable(&ev) {
								//TODO: build filter and replace event
								//TODO: If two events have the same timestamp, the event with the lowest id SHOULD be retained, and the other discarded
								let filter = Filter::new();
								if let Ok(old_ev) = query_events_db(filter) {
									//TODO: check if you should query for multiple replaced events
									write_new_event_db(ev, Some(old_ev)).await;
								}
							} else {
								let ret = write_new_event_db(ev, None).await;
								if ret { ok_events.push(event_id); }
							}
						},
						DbRequest::WriteSub(ns) => { write_new_subscription_db(ns); },
						DbRequest::WriteClient(ct) => { write_new_client_db(ct).await; },
						DbRequest::ReplayEvents { client_id, filters } => { replay_request.push((client_id, filters)); },
						_ => {},
					}
					println!("[CIVKITD] - NOTE PROCESSING: Note processor received DB requests");
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
				let commitment = event_id.as_str();
				let position = self.config.mainstay.position;
				let token = &self.config.mainstay.token;
											
				let req = send_commitment(commitment, position, token, &self.config.mainstay).await.unwrap();

				match req.send().await {
					Ok(_) => println!("Commitment sent successfully"),
					Err(err) => println!("Error sending commitment: {}", err),
				}
				send_db_result_handler_lock.await.send(ok_event);
			}
		}
	}
}
