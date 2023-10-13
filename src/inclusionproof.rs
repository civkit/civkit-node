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

pub struct InclusionProof {
    txid: Arc<Mutex<String>>,
    commitment: Arc<Mutex<String>>,
    merkle_root: Arc<Mutex<String>>,
    config: Config,
}

impl InclusionProof {
	pub fn new(txid: String, commitment: String, merkle_root: String, our_config: Config) -> Self {
        InclusionProof {
            txid: Arc::new(Mutex::new(txid)),
            commitment: Arc::new(Mutex::new(commitment)),
            merkle_root: Arc::new(Mutex::new(merkle_root)),
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
                    
                    self.txid.lock().unwrap().replace(self.txid.lock().unwrap().as_str(), txid);
                    self.commitment.lock().unwrap().replace(self.commitment.lock().unwrap().as_str(), commitment);
                    self.merkle_root.lock().unwrap().replace(self.merkle_root.lock().unwrap().as_str(), merkle_root);
                },
                Err(err) => println!("Error in retrieving inclusion proof: {}", err),
            }
            
			sleep(Duration::from_millis(1000)).await;
        }
    }
}
