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
        }
    }
}