// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.
//
// This module will receive a serialized payload from the BoardManager and
// pack them into an anchor(?) and at periodic intervals, counter-sign them
// with the Board pubkey and submit for validation on the Mainstay server.
// 

use std::sync::Arc;
use std::sync::Mutex;
use bitcoin::secp256k1::{PublicKey};

// use civkit::events;
// use civkit::events::ClientEvents;
// use civkit::kindprocessor::KindProcessor;

pub struct AnchorManager {
    our_service_pubkey: Arc<PublicKey>, 
    our_service_offers: Arc<SerializedOffers>,     
}

pub struct SerializedOffers {
    offers: Vec<SerializedOffer>
}

pub struct SerializedOffer {
    offer: String,
    meta: String
}

impl SerializedOffers {
    pub fn new() -> Self {
        SerializedOffers {
            offers: Vec::new()
        }
    }
    pub fn push(&mut self, serialized_offer: SerializedOffer) -> () {
        self.offers.push(serialized_offer)
    }
}

impl AnchorManager {
	pub fn new(board_pubkey: Arc<PublicKey>) -> Self {
    let offers = SerializedOffers::new();
    AnchorManager {
        our_service_pubkey: board_pubkey,
        our_service_offers: Arc::new(offers)
    }
	}

  pub fn add_payload(&mut self, serialized_offer: Arc<SerializedOffer>) -> () {
    let offer = Arc::clone(&serialized_offer);
    //self.our_service_offers.push(&offer)
  }

	fn commit_note(&self) {}
}

// impl MessageSendKindProvider for AnchorManager {
//       fn get_and_clear_pending_kinds(&self) -> Vec<MessageSendKind> {
//               return (vec![])
//       }
// }
