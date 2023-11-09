// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

/// Simple utilities to query bitcoin RPC API. Inspired by the bitcoin-rpc
/// crate.

use jsonrpc;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Auth {
	None,
	UserPass(String, String)
}

impl Auth {
	pub fn get_user_pass(self) -> Result<(Option<String>, Option<String>), ()> {
		match self {
			Auth::None => Ok((None, None)),
			Auth::UserPass(u, p) => Ok((Some(u), Some(p)))
		}
	}
}

#[derive(Debug)]
pub enum Error {
	JsonRpc(jsonrpc::error::Error),
	InvalidUserPass,
	Json(serde_json::error::Error),
}

pub struct Client {
	client: jsonrpc::client::Client,
}

impl Client {
	pub fn new(url: &str, auth: Auth) -> Result<Self, Error> {
		if let Ok((user, pass)) = auth.get_user_pass() {
			jsonrpc::client::Client::simple_http(url, user, pass)
				.map(|client| Client {
					client,
				})
				.map_err(|e| Error::JsonRpc(e.into()))
		} else { return Err(Error::InvalidUserPass) }
	}

	pub fn call(&self, cmd: &str, args: &[serde_json::Value]) -> Result<(), ()> {
	
		if let Ok(raw_args) = args.iter().map(|a| {
			let json_string = serde_json::to_string(a)?;
			serde_json::value::RawValue::from_string(json_string)
		}).map(|a| a.map_err(|e| Error::Json(e))).collect::<Result<Vec<_>, Error>>() {
	
			let req = self.client.build_request(&cmd, &raw_args);

			println!("req {:?}", req);
		
			let resp = self.client.send_request(req);

			println!("resp {:?}", resp);
		}

		Ok(())
	}
}
