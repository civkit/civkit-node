use reqwest;
use crate::config::Mainstay;
use nostr::{Event, EventId};
use serde_json::json;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use base64::encode;
use bitcoin_hashes::{Hash, sha256};
use crate::nostr_db::{get_cumulative_hash_of_last_event};

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

pub async fn send_commitment(commitment: &str, position: u64, token: &str, config: &Mainstay) -> Result<Request, Box<dyn std::error::Error>> {
    let payload = Payload {
        commitment: commitment.to_string(),
        position: position,
        token: token.to_string(),
    };

    let command = String::from("commitment/send");
    let req = Request::from(Some(&payload), &command, config, None).await?;

    Ok(req)
}

pub async fn get_proof(config: &Mainstay) -> Result<Request, Box<dyn std::error::Error>> {
    let command = format!("commitment/latestproof?position={}", config.position);
    let req = Request::from(None, &command, config, None).await?;

    Ok(req)
}

pub async fn calculate_cumulative_hash(eventId: EventId) -> Vec<u8> {
    let cumulative_hash = get_cumulative_hash_of_last_event().await;

    match cumulative_hash {
        Some(cumulative_hash) => {
            let mut concatenated_hash = cumulative_hash;
            concatenated_hash.extend_from_slice(eventId.as_bytes());
            let cumulative_hash_bytes = sha256::Hash::hash(&concatenated_hash);
            cumulative_hash_bytes.to_vec()
        }
        None => eventId.as_bytes().to_vec(),
    }
}
