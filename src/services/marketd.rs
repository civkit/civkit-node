// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use civkitservice::civkit_service_client::CivkitServiceClient;

use civkitservice::{RegisterRequest, RegisterReply};

use bitcoin::secp256k1::{SecretKey, PublicKey, Secp256k1};
use bitcoin::secp256k1;
use bitcoin::hashes::{Hash, sha256};

use staking_credentials::common::msgs::{AssetProofFeatures, CredentialsFeatures, CredentialPolicy, ServicePolicy, UnsignedCredentialPolicy, UnsignedServicePolicy};


use crate::secp256k1::Message;

use std::str::FromStr;

pub mod civkitservice {
	tonic::include_proto!("civkitservice");
}

fn generate_default_market_policy() -> (CredentialPolicy, ServicePolicy) {

	//TODO: add a Default method in sublibrary ?
	let timestamp = 100;
	let issuance_pubkey = PublicKey::from_str("032e58afe51f9ed8ad3cc7897f634d881fdbe49a81564629ded8156bebd2ffd1af").unwrap();
	let asset_features = Vec::new();
	let asset_proof_features = AssetProofFeatures::new(asset_features);
	let credential_features = Vec::new();
	let credential_proof_features = CredentialsFeatures::new(credential_features);
	let asset_to_credential = 100;
	let expiration_height = 100;

	let unsigned_credential_policy = UnsignedCredentialPolicy {
		timestamp,
		issuance_pubkey,
		asset_proof: asset_proof_features,
		credentials: credential_proof_features,
		asset_to_credential: 100,
		expiration_height: expiration_height,
	};

	let secp_ctx = Secp256k1::new();
	let seckey = [
		59, 148, 11, 85, 134, 130, 61, 253, 2, 174, 59, 70, 27, 180, 51, 107, 94, 203, 174, 253,
		102, 39, 170, 146, 46, 252, 4, 143, 236, 12, 136, 28,
	];
	let seckey = SecretKey::from_slice(&seckey).unwrap();

	//TODO: correct with correct sig
	let msg = b"default credential policy";
	let hash_msg = sha256::Hash::hash(msg);
	let sighash = Message::from_slice(&hash_msg.as_ref()).unwrap();
	let credential_policy_sig = secp_ctx.sign_ecdsa(&sighash, &seckey);

	let credential_policy = CredentialPolicy {
		signature: credential_policy_sig,
		contents: unsigned_credential_policy,
	};

	let timestamp = 100;
	let credential_pubkey = PublicKey::from_str("032e58afe51f9ed8ad3cc7897f634d881fdbe49a81564629ded8156bebd2ffd1af").unwrap();
	let credential_issuers = vec![credential_pubkey];
	let service_ids = vec![100];
	let credentials_to_service = vec![50];
	let expiration_height = 20;

	let unsigned_service_policy = UnsignedServicePolicy {
		timestamp, 
		credential_issuers,
		service_ids,
		credentials_to_service,
		expiration_height,
	};

	let secp_ctx = Secp256k1::new();
	let seckey = [
		59, 148, 11, 85, 134, 130, 61, 253, 2, 174, 59, 70, 27, 180, 51, 107, 94, 203, 174, 253,
		102, 39, 170, 146, 46, 252, 4, 143, 236, 12, 136, 28,
	];
	let seckey = SecretKey::from_slice(&seckey).unwrap();

	//TODO: correct with correct sig
	let msg = b"default service policy";
	let hash_msg = sha256::Hash::hash(msg);
	let sighash = Message::from_slice(&hash_msg.as_ref()).unwrap();
	let service_policy_sig = secp_ctx.sign_ecdsa(&sighash, &seckey);

	let service_policy = ServicePolicy {
		signature: service_policy_sig,
		contents: unsigned_service_policy,
	};

	(credential_policy, service_policy)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

	let mut civkitd_client = CivkitServiceClient::connect(format!("http://[::1]:{}", 50031)).await?;

	let secp_ctx = Secp256k1::new();
	let pubkey = PublicKey::from_secret_key(&secp_ctx, &SecretKey::from_slice(&[42;32]).unwrap());

	let (credential_policy, service_policy) = generate_default_market_policy();

	let request = tonic::Request::new(RegisterRequest {
		service_pubkey: pubkey.serialize().to_vec(),
		credential_policy: vec![],
		service_policy: vec![],
	});

	let response = civkitd_client.register_service(request).await?;

	Ok(())
}
