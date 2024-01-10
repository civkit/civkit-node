// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

//! The ServiceManager responsible to sanitze service kinds note, counter-sign
//! and chain notarize them and dispatch them to peers or clients accordingly.

use bitcoin::BlockHash;
use bitcoin::blockdata::constants::genesis_block;
use bitcoin::network::constants::Network;

use bitcoin::secp256k1::{SecretKey, PublicKey};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::secp256k1;

use civkit::nostr_db::DbRequest;
use civkit::anchormanager::AnchorManager;
use civkit::credentialgateway::CredentialGateway;
use civkit::events::ClientEvents;
use civkit::nodesigner::NodeSigner;
use civkit::peerhandler::PeerInfo;
use civkit::bitcoind_client::BitcoindRequest;
use civkit::config::Config;
use civkit::inclusionproof::InclusionProof;

// use lock from futures::lock
use std::sync::Mutex;
use std::sync::Arc;

use tokio::sync::mpsc;

pub struct ServiceManager
{
	//default_configuration: 
	genesis_hash: BlockHash,

	//TODO: abstract ServiceProcessor, ServiceSigner and AnchorManager in its own Service component ?
	node_signer: Arc<NodeSigner>,
	anchor_manager: Arc<AnchorManager>,

	pub service_events_send: Mutex<mpsc::UnboundedSender<ClientEvents>>,
	pub service_peers_send: Mutex<mpsc::UnboundedSender<PeerInfo>>,

	pub send_db_request: Mutex<mpsc::UnboundedSender<DbRequest>>,

	pub send_bitcoind_request: Mutex<mpsc::UnboundedSender<BitcoindRequest>>,

	pub send_events_gateway: Mutex<mpsc::UnboundedSender<ClientEvents>>,

	our_service_pubkey: PublicKey,
	pub inclusion_proof: Arc<InclusionProof>,
	config: Config,
	secp_ctx: Secp256k1<secp256k1::All>,
}

impl ServiceManager
{
	pub fn new(node_signer: Arc<NodeSigner>, anchor_manager: Arc<AnchorManager>, board_events_send: mpsc::UnboundedSender<ClientEvents>, board_peers_send: mpsc::UnboundedSender<PeerInfo>, send_db_request: mpsc::UnboundedSender<DbRequest>, send_bitcoind_request: mpsc::UnboundedSender<BitcoindRequest>, send_gateway_events: mpsc::UnboundedSender<ClientEvents>, inclusion_proof: Arc<InclusionProof>, our_config: Config) -> Self {
		let secp_ctx = Secp256k1::new();
		let pubkey = PublicKey::from_secret_key(&secp_ctx, &SecretKey::from_slice(&[42;32]).unwrap());
		ServiceManager {
			genesis_hash: genesis_block(our_config.bitcoind_params.chain).header.block_hash(),
			anchor_manager,
			node_signer,
			service_events_send: Mutex::new(board_events_send),
			service_peers_send: Mutex::new(board_peers_send),
			send_db_request: Mutex::new(send_db_request),
			send_bitcoind_request: Mutex::new(send_bitcoind_request),
			send_events_gateway: Mutex::new(send_gateway_events),
			our_service_pubkey: pubkey,
			inclusion_proof: inclusion_proof,
			config: our_config,
			secp_ctx,
		}
	}
}
