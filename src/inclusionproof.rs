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
use serde_json::Value;

use crate::mainstay::{get_proof};
use crate::config::Config;
use crate::nostr_db::{write_new_inclusion_proof_db};

#[derive(Debug, Clone)]
pub struct InclusionProof {
    pub txid: Arc<Mutex<String>>,
    pub commitment: Arc<Mutex<String>>,
    pub merkle_root: Arc<Mutex<String>>,
    pub ops: Arc<Mutex<Vec<Ops>>>,
    pub config: Config,
}

#[derive(Debug)]
pub struct Ops {
    pub append: bool,
    pub commitment: String,
}

impl InclusionProof {
	pub fn new(txid: String, commitment: String, merkle_root: String, ops: Vec<Ops>, our_config: Config) -> Self {
        InclusionProof {
            txid: Arc::new(Mutex::new(txid)),
            commitment: Arc::new(Mutex::new(commitment)),
            merkle_root: Arc::new(Mutex::new(merkle_root)),
            ops: Arc::new(Mutex::new(ops)),
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
                        write_new_inclusion_proof_db(self).await;
                    }
                },
                Err(err) => println!("Error in retrieving inclusion proof: {}", err),
            }
            
			sleep(Duration::from_millis(60 * 1000)).await;
        }
    }
}
