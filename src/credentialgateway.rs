// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

//! The componnent managing the reception of staking credentials and zap
//! notes to ensure notes are not wasting CivKit node ressources.

use bitcoin::BlockHash;
use bitcoin::blockdata::constants::genesis_block;
use bitcoin::network::constants::Network;

use bitcoin::secp256k1::Secp256k1;
use bitcoin::secp256k1;

use nostr::Event;

use staking_credentials::issuance::issuerstate::IssuerState;

use crate::events::ClientEvents;
use crate::bitcoind_client::BitcoindClient;

use tokio::time::{sleep, Duration};
use tokio::sync::mpsc;
use tokio::sync::Mutex;

use std::collections::HashMap;

#[derive(Copy, Clone, Debug)]
struct GatewayConfig {
	//accepted_asset_list: AssetProofFeatures

	//supported_credentials_features: CredentialsFeatures

	/// The number of elements of the credentials cache - Default data struct Merkle Tree
	credentials_consumed_cache_size: u32,
}

impl Default for GatewayConfig {
	fn default() -> GatewayConfig {
		GatewayConfig {
			credentials_consumed_cache_size: 10000000,
		}
	}
}

struct IssuanceManager {
	request_counter: u64,
	table_signing_requests: HashMap<u64, u64> //TODO: add Txid
}

impl IssuanceManager {
	fn register_signing_request(&mut self, client_id: u64, ev: Event) {
		self.table_signing_requests.insert(self.request_counter, client_id);
		self.request_counter += 1;
	}
}

pub struct CredentialGateway {
	bitcoind_client: BitcoindClient,

	genesis_hash: BlockHash,

	default_config: GatewayConfig,

	secp_ctx: Secp256k1<secp256k1::All>,

	receive_credential_event_gateway: Mutex<mpsc::UnboundedReceiver<ClientEvents>>,

	issuance_manager: IssuanceManager,
}

impl CredentialGateway {
	pub fn new(receive_credential_event_gateway: mpsc::UnboundedReceiver<ClientEvents>) -> Self {
		let bitcoind_client = BitcoindClient::new(String::new(), 0, String::new(), String::new());
		let secp_ctx = Secp256k1::new();
		//TODO: should be given a path to bitcoind to use the wallet
		let issuance_manager = IssuanceManager {
			request_counter: 0,
			table_signing_requests: HashMap::new(),
		};
		CredentialGateway {
			bitcoind_client: bitcoind_client,
			genesis_hash: genesis_block(Network::Testnet).header.block_hash(),
			default_config: GatewayConfig::default(),
			secp_ctx,
			receive_credential_event_gateway: Mutex::new(receive_credential_event_gateway),
			issuance_manager: issuance_manager,
		}
	}

	pub async fn run(&mut self) {
		loop {
			sleep(Duration::from_millis(1000)).await;

			let mut credential_queue = Vec::new();
			{
				let mut receive_credential_event_gateway_lock = self.receive_credential_event_gateway.lock();
				if let Ok(credential_event) = receive_credential_event_gateway_lock.await.try_recv() {
					println!("[CIVKITD] - CREDENTIAL: credential received for processing");
					credential_queue.push(credential_event);
				}
			}

			for event in credential_queue {
				match event {
					ClientEvents::Credential { client_id, event } => {
						self.issuance_manager.register_signing_request(client_id, event);
					},
					_ => {},
				}
			}

			{
				//TODO: send txid query to BitcoindClient
			}
		}
	}
}
