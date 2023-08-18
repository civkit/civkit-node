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

use staking_credentials::issuance::issuerstate::IssuerState;

use tokio::time::{sleep, Duration};

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
}

pub struct CredentialGateway {
	genesis_hash: BlockHash,

	default_config: GatewayConfig,

	secp_ctx: Secp256k1<secp256k1::All>,

	issuance_manager: IssuanceManager,
}

impl CredentialGateway {
	pub fn new() -> Self {
		let secp_ctx = Secp256k1::new();
		//TODO: should be given a path to bitcoind to use the wallet
		let issuance_manager = IssuanceManager {};
		CredentialGateway {
			genesis_hash: genesis_block(Network::Testnet).header.block_hash(),
			default_config: GatewayConfig::default(),
			secp_ctx,
			issuance_manager: issuance_manager,
		}
	}

	pub async fn run(&mut self) {
		loop {
			sleep(Duration::from_millis(1000)).await;

			println!("[CIVKITD] - CREDENTIALS: CredentialGateway ready for validation");

			
		}
	}
}
