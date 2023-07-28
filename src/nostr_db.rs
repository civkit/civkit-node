// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use nostr::Event;

use crate::{NostrSub, NostrPeer};

use rusqlite::{Connection, OpenFlags, params};

use std::path::Path;

const CIVKITD_DB_FILE: &str = "civkitd.db";

#[derive(Debug)]
pub enum DbRequest {
	WriteEvent(Event),
	WriteSub(NostrSub),
	DumpEvents,
}

#[derive(Debug)]
struct DbEvent {
	id: i32,
	data: Option<Vec<u8>>,
}

#[derive(Debug)]
struct DbSub {
	sub_id: i32,
	data: Option<Vec<u8>>,
}

pub async fn write_new_event_db(event: Event) {

	//TODO: spawn new thread
	if let Ok(mut conn) = Connection::open_with_flags(
		Path::new(CIVKITD_DB_FILE),
		OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
	) {
		println!("[CIVKITD] - NOTE PROCESSING: Opening database for read / write new event");

		match conn.execute("CREATE TABLE event (
			event_id	INTEGER PRIMARY KEY,
			data		BLOB
		)",
		()) {
			Ok(create) => println!("[CIVKITD] - NOTE PROCESSING: {} rows were updated", create),
			Err(err) => println!("[CIVKITD] - NOTE PROCESSING: table creation failed: {}", err),
		}

		//TODO: add complete event
		let event = DbEvent {
			id: 0,
			data: None,
		};

		match conn.execute("INSERT INTO event (data) VALUES (:data)",
			&[(&event.data)],
		) {
			Ok(update) => println!("[CIVKITD] - NOTE PROCESSING: {} rows were updated", update),
			Err(err) => println!("[CIVKITD] - NOTE PROCESSING: update insert failed: {}", err),
		}

		conn.close().ok();
	} else { println!("Failure to open database"); }
}

pub async fn print_events_db() {

	if let Ok(mut conn) = Connection::open_with_flags(
		Path::new(CIVKITD_DB_FILE),
		OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
	) {
		println!("[CIVKITD] - NOTE PROCESSING: Opening database for read events");

		{
			let mut stmt = conn.prepare("SELECT event_id, data FROM event").unwrap();
			let event_iter = stmt.query_map([], |row| {
				Ok(DbEvent {
					id: row.get(0)?,
					data: row.get(1)?,
				})
			}).unwrap();

			for event in event_iter {
				println!("[CIVKITD] - NOTE PROCESSING: Found event {:?}", event.unwrap());
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

		let event = DbEvent {
			id: 0,
			data: None,
		};

		conn.execute(
			"INSERT INTO event (data) VALUES (:data)",
			&[(&event.data)],
		);
	}
}

//TODO: log function for client
