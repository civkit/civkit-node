use reqwest;
use crate::config::Mainstay;
use nostr::Event;
use serde_json::json;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use base64::encode;
use sha2::{Digest, Sha256};
use hex;

pub struct Request(reqwest::RequestBuilder);

#[derive(Serialize, Deserialize)]
pub struct Payload {
    pub commitment: String,
    pub position: u64,
    pub token: String,
}

impl Request {
    //Construct a request from the given payload and config
    pub async fn from(
        payload: Option<&Payload>,
        command: &String,
        config: &Mainstay,
        signature: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        //Build request
        let client = reqwest::Client::new();
        let url = reqwest::Url::parse(&format!("{}/{}", config.url, command))?;

        //If there is a payload this is a 'POST' request, otherwise a 'GET' request
        let req = match payload {
            Some(p) => {
                let payload_str = String::from(serde_json::to_string(&p)?);
                let payload_enc = encode(payload_str);
                let mut data = HashMap::new();
                data.insert("X-MAINSTAY-PAYLOAD", &payload_enc);

                let mut signature = match signature {
                    Some(s) => s,
                    None => String::from(""),
                };
                data.insert("X-MAINSTAY-SIGNATURE", &signature);
                client
                    .post(url)
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .json(&data)
            }
            None => client
                .get(url)
                .header(reqwest::header::CONTENT_TYPE, "application/json"),
        };

        Ok(Self(req))
    }

    pub async fn send(self) -> std::result::Result<reqwest::Response, reqwest::Error> {
        self.0.send().await
    }
}

pub async fn send_commitment(commitment: &str, position: u64, token: &str, config: &Mainstay) -> Result<(), Box<dyn std::error::Error>> {
    let payload = Payload {
        commitment: commitment.to_string(),
        position: position,
        token: token.to_string(),
    };

    let command = String::from("commitment/send");
    let req = Request::from(Some(&payload), &command, config, None).await?;

    tokio::spawn(async move {
        match req.send().await {
            Ok(_) => println!("Request successful"),
            Err(err) => println!("Error sending request to: {}", err),
        }
    });

    Ok(())
}

pub fn hash_event(event: &Event) -> String {
	let mut hasher = Sha256::new();
	let json = serde_json::to_string(event).unwrap();
  
	hasher.update(json.as_bytes());
	let hash = hasher.finalize();
	let hex_hash = hex::encode(hash.as_slice());

	hex_hash
}
