// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use nostr::{Event, EventBuilder, Filter, Keys, Kind};

use crate::{NostrSub, NostrPeer, NostrClient};

use crate::mainstay::{calculate_cumulative_hash};

use rusqlite::{Connection, OpenFlags, params};

use std::path::Path;

const CIVKITD_DB_FILE: &str = "civkitd.db";

#[derive(Debug)]
pub enum DbRequest {
	WriteEvent(Event),
	WriteSub(NostrSub),
	WriteClient(NostrClient),
	ReplayEvents { client_id: u64, filters: Vec<Filter> },
	DumpEvents,
	DumpClients,
}

#[derive(Debug)]
struct DbEvent {
	id: u32,
	sha256: Vec<u8>,
	pubkey: Vec<u8>,
	timestamp: i64,
	kind: u32,
	content: Option<String>,
	cumulative_hash: Vec<u8>,
}

#[derive(Debug)]
struct DbSub {
	sub_id: i32,
	data: Option<Vec<u8>>,
}

#[derive(Debug)]
struct DbClient {
	client_id: i32,
	data: Option<Vec<u8>>,
}

pub async fn write_new_event_db(event: Event, old_event: Option<Vec<Event>>) -> bool {

	//TODO: spawn new thread
	if let Ok(mut conn) = Connection::open_with_flags(
		Path::new(CIVKITD_DB_FILE),
		OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
	) {
		println!("[CIVKITD] - NOTE PROCESSING: Opening database for read / write new event");

		match conn.execute("CREATE TABLE event (
			event_id			INTEGER PRIMARY KEY,
			sha256				BLOB,
			pubkey				BLOB,
			timestamp			BIG INT,
			kind				UNSIGNED INTEGER,
			content				TEXT,
			cumulative_hash 	BLOB
		)",
		()) {
			Ok(create) => println!("[CIVKITD] - NOTE PROCESSING: {} rows were updated", create),
			Err(err) => println!("[CIVKITD] - NOTE PROCESSING: table creation failed: {}", err),
		}

		//TODO: add complete event
		let event = DbEvent {
			id: 0,
			sha256: event.id.as_bytes().to_vec(),
			pubkey: event.pubkey.serialize().to_vec(),
			timestamp: event.created_at.as_i64(),
			kind: event.kind.as_u32(),
			content: Some(event.content),
			cumulative_hash: calculate_cumulative_hash(event.id).await,
		};

		match conn.execute("INSERT INTO event (sha256, pubkey, timestamp, kind, content, cumulative_hash) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
			(&event.sha256, &event.pubkey, &event.timestamp, &event.kind, &event.content, &event.cumulative_hash),
		) {
			Ok(update) => println!("[CIVKITD] - NOTE PROCESSING: {} rows were updated", update),
			Err(err) => println!("[CIVKITD] - NOTE PROCESSING: update insert failed: {}", err),
		}

		conn.close().ok();
		return true;
	} else { println!("Failure to open database"); }
	return false;
}

pub async fn print_events_db() {

	if let Ok(mut conn) = Connection::open_with_flags(
		Path::new(CIVKITD_DB_FILE),
		OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
	) {
		println!("[CIVKITD] - NOTE PROCESSING: Opening database for read events");

		{
			let mut stmt = conn.prepare("SELECT event_id, sha256, pubkey, timestamp, kind, content, cumulative_hash FROM event").unwrap();
			let event_iter = stmt.query_map([], |row| {
				Ok(DbEvent {
					id: row.get(0)?,
					sha256: row.get(1)?,
					pubkey: row.get(2)?,
					timestamp: row.get(3)?,
					kind: row.get(4)?,
					content: row.get(5)?,
					cumulative_hash: row.get(6)?,
				})
			}).unwrap();

			for event in event_iter {
				println!("[CIVKITD] - NOTE PROCESSING: Found event {:?}", event.unwrap());
			}
		}

		conn.close().ok();
	} else { println!("Failure to open database"); }
}

pub fn query_events_db(filter: Filter) -> Result<Vec<Event>, ()> {

	if let Ok(mut conn) = Connection::open_with_flags(
		Path::new(CIVKITD_DB_FILE),
		OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
	) {
		if let Some(kinds) = filter.kinds {
			//TODO: iter on all the kinds provided by the filter
			let sql = format!("SELECT event_id, sha256, pubkey, timestamp, kind, content, cumulative_hash FROM event WHERE kind = {}", kinds[0].as_u32());
			let mut stmt = conn.prepare(&sql).unwrap();
			let event_iter = stmt.query_map([], |row| {
				Ok(DbEvent {
					id: row.get(0)?,
					sha256: row.get(1)?,
					pubkey: row.get(2)?,
					timestamp: row.get(3)?,
					kind: row.get(4)?,
					content: row.get(5)?,
					cumulative_hash: row.get(6)?,
				})
			}).unwrap();

			let mut result_events = Vec::new();

			//TODO: write keys on DB
			let dummy_keys = Keys::generate();
			for event in event_iter {
				let db_event = event.unwrap();
				let e = EventBuilder::new(Kind::from(db_event.kind as u64), db_event.content.unwrap(), &[]).to_event(&dummy_keys).unwrap();
				result_events.push(e);
			}

			return Ok(result_events);
		}
	}

	Err(())
}

pub async fn get_cumulative_hash_of_last_event() -> Option<Vec<u8>> {
    if let Ok(mut conn) = Connection::open_with_flags(
            Path::new(CIVKITD_DB_FILE),
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        ) {
        let mut stmt = conn
            .prepare("SELECT cumulative_hash FROM event ORDER BY event_id DESC LIMIT 1")
            .unwrap();
		return match stmt.query_row([], |row| row.get(0)) {
			Ok(cumulative_hash) => {
				Some(cumulative_hash)
			},
			Err(_) => None,
		};
    }

    None
}

pub async fn write_new_client_db(client: NostrClient) {

	//TODO: spawn new thread
	if let Ok(mut conn) = Connection::open_with_flags(
		Path::new(CIVKITD_DB_FILE),
		OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
	) {
		println!("[CIVKITD] - NOTE PROCESSING: Opening database for read / write new client");

		match conn.execute("CREATE TABLE client (
			client_id	INTEGER PRIMARY KEY,
			data		BLOB
		)",
		()) {
			Ok(create) => println!("[CIVKITD] - NOTE PROCESSING: {} rows were updated", create),
			Err(err) => println!("[CIVKITD] - NOTE PROCESSING: table creation failed: {}", err),
		}

		let client = DbClient {
			client_id: 0,
			data: None,
		};

		match conn.execute("INSERT INTO client (data) VALUES (:data)",
			&[(&client.data)],
		) {
			Ok(update) => println!("[CIVKITD] - NOTE PROCESSING: {} rows were updated", update),
			Err(err) => println!("[CIVKITD] - NOTE PROCESSING: update insert failed: {}", err),
		}

		conn.close().ok();
	} else { println!("Failure to open database"); }
}

pub async fn print_clients_db() {

	if let Ok(mut conn) = Connection::open_with_flags(
		Path::new(CIVKITD_DB_FILE),
		OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
	) {
		println!("[CIVKITD] - NOTE PROCESSING: Opening database for read clients");

		{
			let mut stmt = conn.prepare("SELECT client_id, data FROM client").unwrap();
			let client_iter = stmt.query_map([], |row| {
				Ok(DbClient {
					client_id: row.get(0)?,
					data: row.get(1)?,
				})
			}).unwrap();

			for client in client_iter {
				println!("[CIVKITD] - NOTE PROCESSING: Found client {:?}", client.unwrap());
			}
		}

		conn.close().ok();
	} else { println!("Failure to open database"); }
}

pub async fn write_new_subscription_db(subscription: NostrSub) {

	if let Ok(conn) = Connection::open_in_memory() {
		conn.execute("CREATE TABLE event (
			sub_id		INTEGER PRIMARY KEY,
			data		BLOB
		)",
		());

		let subscription = DbSub {
			sub_id: 0,
			data: None,
		};

		conn.execute(
			"INSERT INTO event (data) VALUES (:data)",
			&[(&subscription.data)],
		);
	}
}

pub async fn log_new_peer_db(peer: NostrPeer) {

	if let Ok(conn) = Connection::open_in_memory() {
		conn.execute("CREATE TABLE event (
			event_id	INTEGER PRIMARY KEY,
			data		BLOB
		)",
		());

		//let event = DbEvent {
		//	id: 0,
		//	kind: 0,
		//	data: None,
		//};

		//conn.execute(
		//	"INSERT INTO event (data) VALUES (:data)",
		//	&[(&event.data)],
		//);
	}
}

//TODO: log function for client
