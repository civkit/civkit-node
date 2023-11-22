// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use std::sync::Arc;
use std::thread;
use std::sync::Mutex;
use tokio::time::{sleep, Duration};
use serde_json::{Value, from_str, to_value};
use crate::verifycommitment::verify_merkle_root_inclusion;

use crate::mainstay::{get_proof};
use crate::config::Config;
use crate::nostr_db::{write_new_inclusion_proof_db};
use crate::rpcclient::{Client, Auth};

pub struct InclusionProof {
    pub txid: Arc<Mutex<String>>,
    pub commitment: Arc<Mutex<String>>,
    pub merkle_root: Arc<Mutex<String>>,
    pub ops: Arc<Mutex<Vec<Ops>>>,
    pub txoutproof: Arc<Mutex<String>>,
    pub raw_tx: Arc<Mutex<Value>>,
    pub config: Config,
}

pub struct Ops {
    pub append: bool,
    pub commitment: String,
}

impl InclusionProof {
	pub fn new(txid: String, commitment: String, merkle_root: String, ops: Vec<Ops>, txout_proof: String, raw_tx: Value, our_config: Config) -> Self {
        InclusionProof {
            txid: Arc::new(Mutex::new(txid)),
            commitment: Arc::new(Mutex::new(commitment)),
            merkle_root: Arc::new(Mutex::new(merkle_root)),
            ops: Arc::new(Mutex::new(ops)),
            txoutproof: Arc::new(Mutex::new(txout_proof)),
            raw_tx: Arc::new(Mutex::new(raw_tx)),
            config: our_config,
        }
    }

    pub async fn run(&mut self) {
		loop {
            let req = get_proof(&self.config.mainstay).await.unwrap();

            match req.send().await {
                Ok(response) => {

                    let body = response.bytes().await.unwrap();
                    let response_json: Value = serde_json::from_slice(&body).unwrap();
                    let txid = response_json["response"]["txid"].as_str().unwrap();
                    let commitment = response_json["response"]["commitment"].as_str().unwrap();
                    let merkle_root = response_json["response"]["merkle_root"].as_str().unwrap();
                    let ops = response_json["response"]["ops"].as_array().unwrap();

                    if (self.txid.lock().unwrap().as_str() != txid) {
                        *self.txid.lock().unwrap() = txid.to_string();
                        *self.commitment.lock().unwrap() = commitment.to_string();
                        *self.merkle_root.lock().unwrap() = merkle_root.to_string();

                        *self.ops.lock().unwrap() = ops.iter()
                            .map(|value| {
                                let append = value["append"].as_bool().unwrap();
                                let commitment = value["commitment"].as_str().unwrap().to_string();
                                Ops { append, commitment }
                            })
                            .collect();
                        
                        let client = Client::new(format!("{}:{}/", self.config.bitcoind_params.host, self.config.bitcoind_params.port).as_str(),
                            Auth::UserPass(self.config.bitcoind_params.rpc_user.to_string(),
                                self.config.bitcoind_params.rpc_password.to_string())).unwrap();
                        let txid_json_value = to_value(txid).unwrap();
                        let txid_json = Value::Array(vec![txid_json_value]);
                        if let Ok(response) = client.call("gettxoutproof", &[txid_json]) {
                            if let Some(raw_value) = response.result {
                                let mut txout_proof = raw_value.get().to_string();
                                *self.txoutproof.lock().unwrap() = txout_proof;
                            }
                        }

                        if let Ok(response) = client.call("getrawtransaction", &[Value::String(txid.to_string()), Value::Bool(true)]) {
                            if let Some(raw_value) = response.result {
                                let json_value: Value = from_str(raw_value.get()).unwrap();
                                *self.raw_tx.lock().unwrap() = json_value;
                            }
                        }
                        write_new_inclusion_proof_db(self).await;
                    }
                },
                Err(err) => println!("Error in retrieving inclusion proof: {}", err),
            }
            
			sleep(Duration::from_millis(60 * 1000)).await;
        }
    }
}
