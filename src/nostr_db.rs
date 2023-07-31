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

use rusqlite::{Connection, params};

const CIVKITD_DB_FILE: &str = "civkitd.db";

struct DbEvent {
	id: i32,
	data: Option<Vec<u8>>,
}

struct DbSub {
	sub_id: i32,
	data: Option<Vec<u8>>,
}

pub async fn log_new_event_db(event: Event) {

	if let Ok(conn) = Connection::open_in_memory() {
		conn.execute("CREATE TABLE event (
			event_id	INTEGER PRIMARY KEY,
			data		BLOB
		)",
		());

		//TODO: add complete event
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

pub async fn log_new_subscription_db(subscription: NostrSub) {

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

//pub async fn log_new_nostr_client(nostr_client: NostrClient) {
//
//}

//TODO log clients and peers
