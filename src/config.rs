use std::fs;
use toml;
use serde_derive::Deserialize;

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
pub struct Config {
    pub performance: Performance,
    pub spam_protection: SpamProtection,
    pub connections: Connections,
    pub civkitd: Civkitd,
    pub logging: Logging,
    pub mainstay: Mainstay,
    pub bitcoind_params: BitcoindParams,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
pub struct Performance {
    pub max_db_size: i32,
    pub max_event_age: i32,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
pub struct SpamProtection {
    pub requestcredentials: bool,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
pub struct Connections {
    pub maxclientconnections: i32,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
pub struct Civkitd {
    pub network: String,
    pub noise_port: i32,
    pub nostr_port: i32,
    pub cli_port: i32,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
pub struct Logging {
    pub level: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
pub struct Mainstay {
    pub url: String,
    pub position: u64,
    pub token: String,
    pub base_pubkey: String,
    pub chain_code: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
pub struct BitcoindParams {
	pub host: String,
	pub port: String,
	pub rpc_user: String,
	pub rpc_password: String,
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
                url: "http://testnet.mainstay.xyz/api/v1".to_string(),
                position: 4,
                token: "dd7a917e-6cc6-4d95-9eed-cc9ebc20f9a9".to_string(),
                base_pubkey: "021a89d4cdc8559f5aa34894049bcda4b2c648a28d8db29339595114ff3208ead9".to_string(),
                chain_code: "0a090f710e47968aee906804f211cf10cde9a11e14908ca0f78cc55dd190ceaa".to_string(),
            },
	    bitcoind_params: BitcoindParams {
		host: "http://209.250.231.82".to_string(),
		port: "18332".to_string(), // regtest
		rpc_user: "someuser".to_string(),
		rpc_password: "somepass".to_string(),
	    }
        }
    }
}
