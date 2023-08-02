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

// use lock from futures::lock
use std::sync::Mutex;
use std::sync::Arc;

use tokio::sync::mpsc;

pub struct ServiceManager
{
	//default_configuration: 
	genesis_hash: BlockHash,

	credential_gateway: Arc<CredentialGateway>,
	//TODO: abstract ServiceProcessor, ServiceSigner and AnchorManager in its own Service component ?
	node_signer: Arc<NodeSigner>,
	anchor_manager: Arc<AnchorManager>,

	pub service_events_send: Mutex<mpsc::UnboundedSender<ClientEvents>>,
	pub service_peers_send: Mutex<mpsc::UnboundedSender<PeerInfo>>,

	pub send_db_request: Mutex<mpsc::UnboundedSender<DbRequest>>,

	our_service_pubkey: PublicKey,
	secp_ctx: Secp256k1<secp256k1::All>,
}

impl ServiceManager
{
	pub fn new(credential_gateway: Arc<CredentialGateway>, node_signer: Arc<NodeSigner>, anchor_manager: Arc<AnchorManager>, board_events_send: mpsc::UnboundedSender<ClientEvents>, board_peers_send: mpsc::UnboundedSender<PeerInfo>, send_db_request: mpsc::UnboundedSender<DbRequest>) -> Self {
		let secp_ctx = Secp256k1::new();
		let pubkey = PublicKey::from_secret_key(&secp_ctx, &SecretKey::from_slice(&[42;32]).unwrap());
		ServiceManager {
			genesis_hash: genesis_block(Network::Testnet).header.block_hash(),
			credential_gateway,
			anchor_manager,
			node_signer,
			service_events_send: Mutex::new(board_events_send),
			service_peers_send: Mutex::new(board_peers_send),
			send_db_request: Mutex::new(send_db_request),
			our_service_pubkey: pubkey,
			secp_ctx,
		}
	}
}
