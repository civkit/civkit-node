// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

//! The top-level component of a Civ Kit node responsible to sanitize and
//! order trade kinds, counter-sign and anchor them and dispatch them to
//! clients according to requests.

use bitcoin::BlockHash;
use bitcoin::blockdata::constants::genesis_block;
use bitcoin::network::constants::Network;

use bitcoin::secp256k1::{SecretKey, PublicKey};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::secp256k1;

use civkit::events;
use civkit::anchormanager::AnchorManager;

use std::sync::Mutex;
use std::sync::Arc;

pub struct BoardManager
{
	//default_configuration: 
	genesis_hash: BlockHash,

	//credentials_gateway: G,
	//kind_processor: K,
	//node_signer: S,
	anchor_manager: Arc<AnchorManager>,

	our_board_pubkey: PublicKey,
	secp_ctx: Secp256k1<secp256k1::All>,

	//TODO: persistence
	//TODO: kind handler

	pending_kind_events: Mutex<Vec<events::Event>>
}

impl BoardManager
{
	pub fn new(anchor_manager: Arc<AnchorManager>) -> Self {
		let secp_ctx = Secp256k1::new();
		let pubkey = PublicKey::from_secret_key(&secp_ctx, &SecretKey::from_slice(&[42;32]).unwrap());
		BoardManager {
			genesis_hash: genesis_block(Network::Testnet).header.block_hash(),
			anchor_manager: anchor_manager.clone(),
			our_board_pubkey: pubkey,
			secp_ctx,
			pending_kind_events: Mutex::new(Vec::new()),
		}
	}
}
