// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

//! An interface to sanitize and enforce service policy on the received notes.

use crate::nostr_db::DbRequest;
use crate::nostr_db::{write_new_subscription_db, write_new_event_db, write_new_client_db, print_events_db, print_clients_db};

use std::sync::Mutex;

use tokio::sync::mpsc;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration};

pub struct NoteProcessor {
	note_counters: Mutex<u64>,
	current_height: u64,

	receive_db_requests: TokioMutex<mpsc::UnboundedReceiver<DbRequest>>,

	receive_db_requests_manager: TokioMutex<mpsc::UnboundedReceiver<DbRequest>>,
}

impl NoteProcessor {
	pub fn new(receive_db_requests: mpsc::UnboundedReceiver<DbRequest>, receive_db_requests_manager: mpsc::UnboundedReceiver<DbRequest>) -> Self {
		NoteProcessor {
			note_counters: Mutex::new(0),
			current_height: 0,

			receive_db_requests: TokioMutex::new(receive_db_requests),

			receive_db_requests_manager: TokioMutex::new(receive_db_requests_manager),
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
			{
				let mut receive_db_requests_lock = self.receive_db_requests.lock();
				if let Ok(db_request) = receive_db_requests_lock.await.try_recv() {
					match db_request {
						DbRequest::WriteEvent(ev) => { write_new_event_db(ev).await; },
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

			//TODO: visitor on DB with query	
		}
	}
}
