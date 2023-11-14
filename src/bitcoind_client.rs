// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use crate::config::Config;
use crate::rpcclient::{Client, Auth};

use jsonrpc::Response;

use bitcoin::{MerkleBlock, Txid};
use bitcoin_hashes::hex::FromHex;

use tokio::sync::{mpsc, oneshot};
use tokio::sync::Mutex as TokioMutex;

use tokio::time::{sleep, Duration};

use crate::inclusionproof::InclusionProof;
use crate::verifycommitment::{verify_commitments, verify_slot_proof, verify_merkle_root_inclusion};
use crate::nostr_db::get_hashes_of_all_events;

#[derive(Debug)]
pub enum BitcoindRequest {
	CheckRpcCall,
	GenerateTxInclusionProof { txid: String, respond_to: oneshot::Sender<Option<String>> },
}

pub struct BitcoindClient {
	host: String,
	port: String,
	rpc_user: String,
	rpc_password: String,
}

impl BitcoindClient {
	pub fn new(host: String, port: String, rpc_user: String, rpc_password: String) -> Self {
		BitcoindClient {
			host: host,
			port: port,
			rpc_user: rpc_user,
			rpc_password: rpc_password,
		}
	}

	pub async fn gettxoutproof() {

	}

	pub async fn verifytxoutproof(txid: String, slot: usize, mut inclusion_proof: InclusionProof) -> bool {
		let event_commitments = get_hashes_of_all_events().await.unwrap();
		if !verify_commitments(event_commitments, &mut inclusion_proof) {
			return false;
		}
		if !verify_slot_proof(slot, &mut inclusion_proof) {
			return false;
		}
		if !verify_merkle_root_inclusion(txid, &mut inclusion_proof) {
			return false;
		}

		true
	}

	//TODO: run and dispatch call to bitcoind
}

pub struct BitcoindHandler {

	receive_bitcoind_request: TokioMutex<mpsc::UnboundedReceiver<BitcoindRequest>>,

	bitcoind_client: BitcoindClient,

	rpc_client: Client,

	config: Config,
}

impl BitcoindHandler {
	pub fn new(config: Config, receive_bitcoind_requests: mpsc::UnboundedReceiver<BitcoindRequest>) -> BitcoindHandler {

		let bitcoind_client = BitcoindClient {
			host: config.bitcoind_params.host.clone(),
			port: config.bitcoind_params.port.clone(),
			rpc_user: config.bitcoind_params.rpc_user.clone(),
			rpc_password: config.bitcoind_params.rpc_password.clone(),
		};

		let separator = ":";
		let url = bitcoind_client.host.clone() + &separator + &bitcoind_client.port.clone();

		let user_pass = Auth::UserPass(bitcoind_client.rpc_user.clone(), bitcoind_client.rpc_password.clone());

		let rpc_client = Client::new(&url, user_pass).unwrap();

		BitcoindHandler {
			receive_bitcoind_request: TokioMutex::new(receive_bitcoind_requests),
			bitcoind_client,
			rpc_client,
			config,
		}
	}

	pub async fn run(&mut self) {
		loop {
			sleep(Duration::from_millis(1000)).await;

			let mut receive_bitcoind_request_lock = self.receive_bitcoind_request.lock();
			if let Ok(bitcoind_request) = receive_bitcoind_request_lock.await.try_recv() {
				match bitcoind_request {
					BitcoindRequest::CheckRpcCall => {
						println!("[CIVKITD] - BITCOIND CLIENT: Received rpc call - Test bitcoind");
 
						self.rpc_client.call("getblockchaininfo", &vec![]);
					},
					BitcoindRequest::GenerateTxInclusionProof { txid, respond_to } => {
						println!("[CIVKITD] - BITCOIND CLIENT: Received rpc call - Generate merkle block");

						let txid_json_value = serde_json::to_value(txid).unwrap();
						let txid_json = serde_json::Value::Array(vec![txid_json_value]);

						if let Ok(response) = self.rpc_client.call("gettxoutproof", &[txid_json]) {
							if let Some(raw_value) = response.result {
								let mut mb_string = raw_value.get().to_string();
								let index = mb_string.find('\"').unwrap();
								mb_string.remove(index);
								let index = mb_string.find('\"').unwrap();
								mb_string.remove(index);
								//let mb_bytes = Vec::from_hex(&mb_string).unwrap();
								//let mb: MerkleBlock = bitcoin::consensus::deserialize(&mb_bytes).unwrap();
								respond_to.send(Some(mb_string));
							}
						} else { respond_to.send(None); }
					},
					_ => {},
				}
			}
		}
	}
}
