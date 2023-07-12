// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use lightning::onion_message::OnionMessenger;
use lightning::sign::KeysManager;
use lightning::ln::peer_handler::IgnoringMessageHandler;
use lightning::util::logger::{Logger, Record};

use bitcoin::secp256k1::{SecretKey, PublicKey};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::secp256k1;

use std::sync::Arc;
use std::time::Duration;
use std::thread;

struct FakeLogger;
impl Logger for FakeLogger {
	fn log(&self, record: &Record) { unimplemented!() }
}

pub struct OnionBox {

	//TODO: add OnionMessenger

	our_node_pubkey: PublicKey,
	secp_ctx: Secp256k1<secp256k1::All>
}

impl OnionBox {
	pub fn new() -> Self {
		let secp_ctx = Secp256k1::new();
		let pubkey = PublicKey::from_secret_key(&secp_ctx, &SecretKey::from_slice(&[42;32]).unwrap());

		let seed = [42u8; 32];
		let time = Duration::from_secs(123456);
		let keys_manager = KeysManager::new(&seed, time.as_secs(), time.subsec_nanos());
		let keys_manager_2 = KeysManager::new(&seed, time.as_secs(), time.subsec_nanos());
		let logger = Arc::new(FakeLogger {});
		let ignoring_message_handler = IgnoringMessageHandler {};

		//let onion_messenger = OnionMessenger::new(&keys_manager, &keys_manager_2, logger, &ignoring_message_handler);

		OnionBox {
			our_node_pubkey: pubkey,
			secp_ctx,
		}
	}

	pub async fn run(&self) {
		loop {
			let one_second = Duration::from_secs(1);

			thread::sleep(one_second);
			//TODO: receive onion messages and send them to the CredentialGateway
		}
	}
}
