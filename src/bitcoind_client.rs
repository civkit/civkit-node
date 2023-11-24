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

use bitcoin::consensus::serialize;
use bitcoin::{MerkleBlock, Txid};
use bitcoin_hashes::hex::{ToHex, FromHex};

use staking_credentials::common::utils::Proof;

use tokio::sync::{mpsc, oneshot};
use tokio::sync::Mutex as TokioMutex;

use tokio::time::{sleep, Duration};

use crate::inclusionproof::InclusionProof;
use crate::verifycommitment::{verify_merkle_root_inclusion};

#[derive(Debug)]
pub enum BitcoindRequest {
	CheckRpcCall,
	GenerateTxInclusionProof { txid: String, respond_to: oneshot::Sender<Option<String>> },
	CheckMerkleProof { request_id: u64, proof: Proof },
	VerifyInclusionProof { inclusion_proof: InclusionProof, respond_to: oneshot::Sender<Option<String>> },
}

#[derive(Debug)]
pub enum BitcoindResult {
	ProofValid { request_id: u64, valid: bool },
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

	pub async fn verifytxoutproof(mut inclusion_proof: InclusionProof) -> bool {
		return verify_merkle_root_inclusion(&mut inclusion_proof);
	}

	//TODO: run and dispatch call to bitcoind
}

pub struct BitcoindHandler {

	receive_bitcoind_request: TokioMutex<mpsc::UnboundedReceiver<BitcoindRequest>>,

	receive_bitcoind_request_gateway: TokioMutex<mpsc::UnboundedReceiver<BitcoindRequest>>,

	send_bitcoind_result_handler: TokioMutex<mpsc::UnboundedSender<BitcoindResult>>,

	bitcoind_client: BitcoindClient,

	rpc_client: Client,

	config: Config,
}

impl BitcoindHandler {
	pub fn new(config: Config, receive_bitcoind_requests: mpsc::UnboundedReceiver<BitcoindRequest>, receive_bitcoind_request_gateway: mpsc::UnboundedReceiver<BitcoindRequest>, send_bitcoind_result_handler: mpsc::UnboundedSender<BitcoindResult>) -> BitcoindHandler {

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
			receive_bitcoind_request_gateway: TokioMutex::new(receive_bitcoind_request_gateway),
			send_bitcoind_result_handler: TokioMutex::new(send_bitcoind_result_handler),
			bitcoind_client,
			rpc_client,
			config,
		}
	}

	pub async fn run(&mut self) {
		loop {
			sleep(Duration::from_millis(1000)).await;

			{
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

			let mut validation_result = Vec::new();
			{
				let mut receive_bitcoind_request_gateway = self.receive_bitcoind_request_gateway.lock();
				if let Ok(bitcoind_request) = receive_bitcoind_request_gateway.await.try_recv() {
					match bitcoind_request {
						BitcoindRequest::CheckMerkleProof { request_id, proof } => {
							println!("[CIVKITD] - BITCOIND CLIENT: Received rpc call - Check merkle proof");

							match proof {
								Proof::MerkleBlock(merkle_block) => {
									let hex_string = serialize(&merkle_block).to_hex();
									let proof_json = serde_json::Value::String(hex_string);

									//TODO: verify transaction paid the correct amount to the correct scrippubkey and deliver credential in function ?
									if let Ok(response) = self.rpc_client.call("verifytxoutproof", &[proof_json]) {
										println!("got an answer {:?}", response);
										if let Some(raw_value) = response.result {
											println!("raw value {}", raw_value);
											let txid_array = raw_value.get();
											if txid_array.len() > 0 {
												println!("[CIVKITD] - BITCOIND CLIENT: Check - Valid proof");
												validation_result.push(BitcoindResult::ProofValid { request_id, valid: true });
											}
										}
									} else { println!("[CIVKITD] - No reply from bitcoind"); }
								},
								_ => { validation_result.push(BitcoindResult::ProofValid { request_id, valid: false }); }
							}
						},
						BitcoindRequest::VerifyInclusionProof { inclusion_proof, respond_to } => {
							println!("[CIVKITD] - BITCOIND CLIENT: Received rpc call - Verify inclusion proof");
	
							let res = BitcoindClient::verifytxoutproof(inclusion_proof).await;
							respond_to.send(Some(res.to_string()));
						}
						_ => {},
					}
				}
			}

			{
				for result in validation_result {
					let mut send_bitcoind_result_handler_lock = self.send_bitcoind_result_handler.lock();
					send_bitcoind_result_handler_lock.await.send(result);
				}
			}
		}
	}
}
