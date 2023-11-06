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

use tokio::sync::mpsc;
use tokio::sync::Mutex as TokioMutex;

use tokio::time::{sleep, Duration};

#[derive(Debug)]
pub enum BitcoindRequest {
	CheckRpcCall,
}

pub struct BitcoindClient {
	host: String,
	port: u16,
	rpc_user: String,
	rpc_password: String,
}

impl BitcoindClient {
	pub fn new(host: String, port: u16, rpc_user: String, rpc_password: String) -> Self {
		BitcoindClient {
			host: host,
			port: port,
			rpc_user: rpc_user,
			rpc_password: rpc_password,
		}
	}

	pub async fn gettxoutproof() {

	}

	pub async fn verifytxoutproof() {

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
			host: "".to_string(),
			port: 0,
			rpc_user: "".to_string(),
			rpc_password: "".to_string(),
		};

		let rpc_client = Client::new(&bitcoind_client.host, Auth::None).unwrap();

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
						println!("[CIVKITD] - BITCOIND CLIENT: Received rpc call");
 
						//TODO: pass real arguments
						self.rpc_client.call("test", &vec![]);
					}
					_ => {},
				}
			}
		}
	}
}
