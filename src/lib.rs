// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use bitcoin::secp256k1::PublicKey;

use nostr::{SubscriptionId, Filter};
use nostr::key::XOnlyPublicKey;

use std::collections::HashMap;
use std::net::SocketAddr;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NostrSub {
	our_side_id: u64,
	id: SubscriptionId,
	filters: Vec<Filter>
}

impl NostrSub {
	pub fn new(our_side_id: u64, id: SubscriptionId, filters: Vec<Filter>) -> Self {
		NostrSub {
			our_side_id,
			id,
			filters,
		}
	}

	pub fn is_our_id(&self, id: &SubscriptionId) -> bool {
		self.id == *id
	}

	pub fn get_filters(&self) -> &Vec<Filter> {
		&self.filters
	}
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NostrPeer {
	peer_pubkey: PublicKey,
}

impl NostrPeer {
	pub fn new(peer_pubkey: PublicKey) -> Self {
		NostrPeer {
			peer_pubkey,
		}
	}
}


//TODO: implement config maxconnections
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct NostrClient {
	//TODO: check we're using Schnorr not ECDSA
	pub pubkey: Option<XOnlyPublicKey>,
	pub client_id: u64,
	pub associated_socket: SocketAddr,

	pub subscriptions: HashMap<u64, ()>,
}

impl NostrClient {
	fn new(client_id: u64, socket: SocketAddr) -> Self {
		NostrClient {
			pubkey: None,
			client_id,
			associated_socket: socket,
			subscriptions: HashMap::new(),
		}
	}

	fn has_pubkey(&self) -> bool {
		self.pubkey.is_some()
	}

	fn add_pubkey(&mut self, pubkey: XOnlyPublicKey) {
		self.pubkey = Some(pubkey);
	}

	fn add_sub(&mut self, sub_id: u64, max_sub: u64) -> bool {
		if self.subscriptions.len() as u64 <= max_sub {
			return self.subscriptions.insert(sub_id, ()).is_none();
		}
		false
	}

	fn has_sub(&self, sub_id: u64) -> bool {
		self.subscriptions.get(&sub_id).is_some()
	}
}

pub mod events;
pub mod nostr_db;
pub mod anchormanager;
pub mod credentialgateway;
pub mod kindprocessor;
pub mod nodesigner;
pub mod oniongateway;
pub mod peerhandler;
pub mod clienthandler;
pub mod config;
pub mod util;
pub mod bitcoind_client;
pub mod mainstay;
pub mod inclusionproof;
