// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use crate::config::Config;

use tokio::sync::mpsc;
use tokio::sync::Mutex as TokioMutex;

#[derive(Debug)]
pub enum BitcoindRequest {
	CheckRpcCall,
}

pub struct BitcoindHandler {

	receive_bitcoind_request: TokioMutex<mpsc::UnboundedReceiver<BitcoindRequest>>,

	bitcoind_client: BitcoindClient,

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

		BitcoindHandler {
			receive_bitcoind_request: TokioMutex::new(receive_bitcoind_requests),
			bitcoind_client,
			config,
		}
	}
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
}
