// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use serde_derive::Deserialize;

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
pub struct BitcoindClient {
	pub host: String,
	pub port: u16,
	pub rpc_user: String,
	pub rpc_password: String,
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
