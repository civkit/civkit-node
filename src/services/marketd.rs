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

pub mod civkitservice {
	tonic::include_proto!("civkitservice");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

	let mut civkitd_client = CivkitServiceClient::connect(format!("http://[::1]:{}", 50031)).await?;

	let secp_ctx = Secp256k1::new();
	let pubkey = PublicKey::from_secret_key(&secp_ctx, &SecretKey::from_slice(&[42;32]).unwrap());

	let request = tonic::Request::new(RegisterRequest {
		service_pubkey: pubkey.serialize().to_vec()
	});

	let response = civkitd_client.register_service(request).await?;

	Ok(())
}
