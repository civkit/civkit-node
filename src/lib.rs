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

#[derive(Clone, PartialEq, Eq)]
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

#[derive(Clone, PartialEq, Eq)]
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
