use std::fs;
use bitcoin::{Block, BlockHeader, Network};
use serde::Serializer;
use toml;
use serde_derive::{Deserialize, Serialize};


#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct Config {
    pub performance: Performance,
    pub spam_protection: SpamProtection,
    pub connections: Connections,
    pub civkitd: Civkitd,
    pub logging: Logging,
    pub mainstay: Mainstay,
    pub bitcoind_params: BitcoindParams,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct Performance {
    pub max_db_size: i32,
    pub max_event_age: i32,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct SpamProtection {
    pub requestcredentials: bool,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct Connections {
    pub maxclientconnections: i32,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct Civkitd {
    pub network: String,
    pub noise_port: i32,
    pub nostr_port: i32,
    pub cli_port: i32,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct Logging {
    pub level: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct Mainstay {
	pub url: String,
	pub position: i32,
	pub token: String,
	pub base_pubkey: String,
	pub chain_code: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct BitcoindParams {
	pub host: String,
	pub port: String,
	pub rpc_user: String,
	pub rpc_password: String,
    pub chain: bitcoin::Network,
}

// default config to fallback
impl Default for Config {
    fn default() -> Self {
        Config {
            performance: Performance {
                max_db_size: 10000,
                max_event_age: 3600,
            },
            spam_protection: SpamProtection {
                requestcredentials: true,
            },
            connections: Connections {
                maxclientconnections: 100,
            },
            civkitd: Civkitd {
                network: "testnet".to_string(),
                noise_port: 9735,
                nostr_port: 50021,
                cli_port: 50031,
            },
            logging: Logging {
                level: "info".to_string(),
            },
            mainstay: Mainstay {
                url: "https://mainstay.xyz/api/v1".to_string(),
                position: 0,
                token: "14b2b754-5806-4157-883c-732baf88849c".to_string(),
		base_pubkey: "038695a7bf3a49d951d7e71bb0ca54158ca1a020e209653706c0dcad344f9b9d05".to_string(),
		chain_code: "14df7ece79e83f0f479a37832d770294014edc6884b0c8bfa2e0aaf51fb00229".to_string(),
            },
	    bitcoind_params: BitcoindParams {
		host: "https://127.0.0.1".to_string(),
		port: "18443".to_string(), // regtest
		rpc_user: "civkitd_client".to_string(),
		rpc_password: "hello_world".to_string(),
        chain: bitcoin::Network::Testnet,
	    }
        }
    }
}
