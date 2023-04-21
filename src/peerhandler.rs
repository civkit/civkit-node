// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

//! The NoiseGateway responsible to handle BOLT8 peers connections.

use bitcoin::secp256k1::{SecretKey, PublicKey};
use bitcoin::secp256k1::Secp256k1;

use lightning::sign::{NodeSigner, KeysManager};
use lightning::ln::peer_handler::{PeerManager, MessageHandler, IgnoringMessageHandler, ErroringMessageHandler, SocketDescriptor as LnSocketTrait, CustomMessageHandler};
use lightning::ln::msgs::{ChannelMessageHandler, RoutingMessageHandler, OnionMessageHandler};
use lightning::util::logger::{Logger, Record};

use lightning_net_tokio::SocketDescriptor;

use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use std::thread::sleep;

use tokio::sync::mpsc;

pub struct PeerInfo {
	pub local_port: u64,
}

impl PeerInfo {
	pub fn new(local_port: u64) -> Self {
		PeerInfo {
			local_port,
		}
	}
}

pub struct FakeLogger;
impl Logger for FakeLogger {
	fn log(&self, record: &Record) { println!("got record dunno what about!") }
}

pub struct NoiseGateway {
	pub peer_manager: Arc<PeerManager<SocketDescriptor, Arc<ErroringMessageHandler>, Arc<IgnoringMessageHandler>, IgnoringMessageHandler, Arc<FakeLogger>, IgnoringMessageHandler, Arc<KeysManager>>>,

	gateway_receive: Mutex<mpsc::UnboundedReceiver<PeerInfo>>,
}

impl NoiseGateway {
	pub fn new(gateway_receive: mpsc::UnboundedReceiver<PeerInfo>) -> Self {
		let secp_ctx = Secp256k1::new();
		let pubkey = PublicKey::from_secret_key(&secp_ctx, &SecretKey::from_slice(&[42;32]).unwrap());
		let ephemeral_bytes = [1 as u8; 32];
		let logger = Arc::new(FakeLogger {});

		let seed = [42u8; 32];
		let time = Duration::from_secs(123456);
		let keys_manager = Arc::new(KeysManager::new(&seed, time.as_secs(), time.subsec_nanos()));

		let chan_handler = ErroringMessageHandler::new();
		let route_handler = IgnoringMessageHandler {};
		let onion_message_handler = IgnoringMessageHandler {};
		let custom_message_handler = IgnoringMessageHandler {};

		let msg_handler = MessageHandler {
			chan_handler: Arc::new(chan_handler),
			route_handler: Arc::new(route_handler),
			onion_message_handler,
			custom_message_handler
		};

		let peer_manager = Arc::new(PeerManager::new(msg_handler, 0, &ephemeral_bytes, logger, keys_manager));

		NoiseGateway {
			peer_manager,
			gateway_receive: Mutex::new(gateway_receive),
		}
	}

	pub async fn run(&self) {
		loop {
			let one_second = Duration::from_secs(1);

			sleep(one_second);

			let mut local_port = 0;
			{
				let mut gateway_receive_lock = self.gateway_receive.lock().unwrap();

				if let Ok(peer) = gateway_receive_lock.try_recv() {
					local_port = peer.local_port;
				}
			}
			if local_port > 0 {
				let peer_mngr = self.peer_manager.clone();
				let secp_ctx = Secp256k1::new();
				let pubkey = PublicKey::from_secret_key(&secp_ctx, &SecretKey::from_slice(&[42;32]).unwrap());
				let peer_addr = format!("[::1]:{}", local_port).parse().unwrap();
				println!("[CIVKITD] - NOISE: opening outgoing noise connection!");
				lightning_net_tokio::connect_outbound(Arc::clone(&peer_mngr), pubkey, peer_addr).await;
			}
		}
	}
}
