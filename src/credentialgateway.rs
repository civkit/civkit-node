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

use bitcoin::{BlockHash, Txid};
use bitcoin::blockdata::constants::genesis_block;
use bitcoin::hashes::{sha256d, Hash, HashEngine};
use bitcoin::network::constants::Network;

use bitcoin::secp256k1::{PublicKey, SecretKey, Secp256k1};
use bitcoin::secp256k1;

use nostr::{Event, Kind, Tag, TagKind};

use staking_credentials::common::msgs::{AssetProofFeatures, CredentialsFeatures, CredentialPolicy, ServicePolicy};
use staking_credentials::common::utils::Proof;
use staking_credentials::issuance::issuerstate::IssuerState;

use staking_credentials::common::msgs::{CredentialAuthenticationResult, CredentialAuthenticationPayload, Decodable, ServiceDeliveranceResult, FromHex};
use staking_credentials::common::utils::Credentials;

use crate::events::ClientEvents;
use crate::bitcoind_client::{BitcoindClient, BitcoindRequest, BitcoindResult};

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

struct IssuanceRequest {
	client_id: u64,
	pending_credentials: Vec<Credentials>,
}

#[derive(Debug)]
enum IssuanceError {
	InvalidDataCarrier,
	Parse,
	Policy,
}

const MAX_CREDENTIALS_PER_REQUEST: usize = 100;

//TODO: protect denial-of-service from client id requests congestion rate
struct IssuanceManager {
	request_counter: u64,
	table_signing_requests: HashMap<u64, IssuanceRequest>,

	issuance_engine: IssuerState,
}

impl IssuanceManager {
	fn register_authentication_request(&mut self, client_id: u64, ev: Event) -> Result<(u64, Proof), IssuanceError> {
		let request_id = self.request_counter;

		if ev.tags.len() != 1 {
			return Err(IssuanceError::InvalidDataCarrier);
		}
		let credential_hex = match &ev.tags[0] {
			Tag::Credential(credential) => { credential },
			_ => { return Err(IssuanceError::InvalidDataCarrier); },
		};
		let credential_msg_bytes = Vec::from_hex(&credential_hex).unwrap();
		let credential_authentication = if let Ok(credential_authentication) = CredentialAuthenticationPayload::decode(&credential_msg_bytes) {
			credential_authentication
		} else { return Err(IssuanceError::Parse); };

		if credential_authentication.credentials.len() > MAX_CREDENTIALS_PER_REQUEST {
			return Err(IssuanceError::Policy);
		}

		self.table_signing_requests.insert(self.request_counter, IssuanceRequest { client_id, pending_credentials: credential_authentication.credentials });
		self.request_counter += 1;

		Ok((request_id, credential_authentication.proof))
	}

	fn validate_authentication_request(&mut self, request_id: u64, result: bool) -> Result<CredentialAuthenticationResult, ()> {
		if let Some(request) = self.table_signing_requests.get(&request_id) {
			//if let Ok(self.issuer_state.authenticate_credentials(request);

			let signatures = vec![];

			let mut credential_authentication_result = CredentialAuthenticationResult::new(signatures);

			return Ok(credential_authentication_result)
		}
		Err(())
	}
	fn get_client_id(&self, request_id: u64) -> u64 {
		if let Some(issuance_request) = self.table_signing_requests.get(&request_id) {
			issuance_request.client_id
		} else { 0 }
	}

}

struct RedemptionManager { }

impl RedemptionManager {
	fn validate_service_deliverance(&mut self, client_id: u64, ev: Event) -> Result<ServiceDeliveranceResult, ()> {

		let service_id = 0;
		let ret = false;
		let reason = vec![];

		let mut service_deliverance_result = ServiceDeliveranceResult::new(service_id, ret, reason);

		Ok(service_deliverance_result)
	}
}

#[derive(Clone)]
struct Service {
	credential_policy: CredentialPolicy,
	service_policy: ServicePolicy,
	registration_height: u64,
}

pub struct CredentialGateway {
	bitcoind_client: BitcoindClient,

	genesis_hash: BlockHash,

	default_config: GatewayConfig,

	secp_ctx: Secp256k1<secp256k1::All>,

	receive_credential_event_gateway: Mutex<mpsc::UnboundedReceiver<ClientEvents>>,
	send_credential_events_gateway: Mutex<mpsc::UnboundedSender<ClientEvents>>,

	send_bitcoind_request_gateway: Mutex<mpsc::UnboundedSender<BitcoindRequest>>,
	receive_bitcoind_result_handler: Mutex<mpsc::UnboundedReceiver<BitcoindResult>>,

	receive_events_gateway: Mutex<mpsc::UnboundedReceiver<ClientEvents>>,

	issuance_manager: IssuanceManager,
	redemption_manager: RedemptionManager,

	hosted_services: HashMap<PublicKey, Service>,

	chain_height: u64,
}

impl CredentialGateway {
	pub fn new(receive_credential_event_gateway: mpsc::UnboundedReceiver<ClientEvents>, send_credential_events_gateway: mpsc::UnboundedSender<ClientEvents>, send_bitcoind_request_gateway: mpsc::UnboundedSender<BitcoindRequest>, receive_bitcoind_result_gateway: mpsc::UnboundedReceiver<BitcoindResult>, receive_events_gateway: mpsc::UnboundedReceiver<ClientEvents>) -> Self {
		let bitcoind_client = BitcoindClient::new(String::new(), "0".to_string(), String::new(), String::new());
		let secp_ctx = Secp256k1::new();
		//TODO: should be given a path to bitcoind to use the wallet

		let secp_ctx = Secp256k1::new();
		let pubkey = PublicKey::from_secret_key(&secp_ctx, &SecretKey::from_slice(&[42;32]).unwrap());

		let asset_proof_features = AssetProofFeatures::new(vec![]);
		let credentials_features = CredentialsFeatures::new(vec![]);

		let issuer_state = IssuerState::new(asset_proof_features, credentials_features, pubkey);

		let issuance_manager = IssuanceManager {
			request_counter: 0,
			table_signing_requests: HashMap::new(),
			issuance_engine: issuer_state,
		};

		let redemption_manager = RedemptionManager {

		};

		let hosted_services = HashMap::new();

		CredentialGateway {
			bitcoind_client: bitcoind_client,
			genesis_hash: genesis_block(Network::Testnet).header.block_hash(),
			default_config: GatewayConfig::default(),
			secp_ctx,
			receive_credential_event_gateway: Mutex::new(receive_credential_event_gateway),
			send_credential_events_gateway: Mutex::new(send_credential_events_gateway),
			send_bitcoind_request_gateway: Mutex::new(send_bitcoind_request_gateway),
			receive_bitcoind_result_handler: Mutex::new(receive_bitcoind_result_gateway),
			receive_events_gateway: Mutex::new(receive_events_gateway),
			issuance_manager: issuance_manager,
			redemption_manager: redemption_manager,
			hosted_services: hosted_services,
			chain_height: 0,
		}
	}

	fn announce_new_service(&self, since: u64) -> Vec<Service> {
		let mut to_be_announced_services = Vec::new();

		for (_, service) in self.hosted_services.iter() {
			if service.registration_height >= since {
				to_be_announced_services.push((*service).clone());
			}
		}
		to_be_announced_services
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

			let mut proofs_to_verify = Vec::new();
			//TODO: change serialization of credential message from bytes payload to encompass ServiceDelivereRequest.
			//let mut deliverance_result_queue = Vec::new();
			for event in credential_queue {
				match event {
					ClientEvents::Credential { client_id, event } => {
						match event.tags[0].kind() {
							//TODO: decode and check the exact credential requested from client
							TagKind::Credential => {
								match self.issuance_manager.register_authentication_request(client_id, event) {
									Ok(proof) => {
										println!("[CIVKITD] - CREDENTIAL: adding a merkle block proof to verify");
										proofs_to_verify.push(proof);
									},
									Err(error) => {
										println!("[CIVKITD] - CREDENTIAL: authentication request error {:?}", error); 
									}
								}
							},
							_ => { println!("[CIVKITD] - CREDENTIAL: credential event error: unknown kind"); }
						}
					},
					_ => {},
				}
			}

			for (request_id, proof) in proofs_to_verify {
				let mut send_bitcoind_request_lock = self.send_bitcoind_request_gateway.lock();
				println!("[CIVKITD] - CREDENTIAL: credential check merkle proof");
				send_bitcoind_request_lock.await.send(BitcoindRequest::CheckMerkleProof { request_id, proof });
			}


			let mut validated_requests = Vec::new();
			{
				let mut receive_bitcoind_result_handler_lock = self.receive_bitcoind_result_handler.lock();
				if let Ok(bitcoind_result) = receive_bitcoind_result_handler_lock.await.try_recv() {
					match bitcoind_result {
						BitcoindResult::ProofValid { request_id, valid } => {
							validated_requests.push((request_id, valid));
						},
						_ => { println!("[CIVKITD] - CREDENTIAL: uncorrect Bitcoin backend result"); },
					}
				}
			}

			let mut authentication_result_queue = Vec::new();
			for (request_id, validation_result) in validated_requests {
				if let Ok(result) = self.issuance_manager.validate_authentication_request(request_id, validation_result) {
					let client_id = self.issuance_manager.get_client_id(request_id);
					authentication_result_queue.push((client_id, result));
				}
			}

			{
				for result in authentication_result_queue {
					let mut send_credential_lock = self.send_credential_events_gateway.lock();
					//TODO: send back event
				}
			}

			let mut service_registration_request = Vec::new();
			{
				let mut receive_events_gateway_lock = self.receive_events_gateway.lock();
				if let Ok(service_registration) = receive_events_gateway_lock.await.try_recv() {
					println!("[CIVKITD] - CREDENTIAL: service registration received for processing");
					service_registration_request.push(service_registration);
				}
			}

			// We register civkit services hosted by this credential gateway
			for service in service_registration_request {
				match service {
					ClientEvents::ServiceRegistration { pubkey, credential_policy, service_policy } => {
						self.hosted_services.insert(pubkey, Service { credential_policy, service_policy, registration_height: self.chain_height });
					},
					_ => { }
				}
			}

			//TODO: announce back new policy to the clients
		}
	}
}
