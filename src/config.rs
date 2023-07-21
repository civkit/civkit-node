use std::fs;
use toml;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub performance: Performance,
    pub spam_protection: SpamProtection,
    pub connections: Connections,
    pub civkitd: Civkitd,
}

#[derive(Debug, Deserialize)]
pub struct Performance {
    pub par: i32,
    pub dbcache: i32,
    pub blocksonly: i32,
    pub maxuploadtarget: i32,
    pub mempoolexpiry: i32,
    pub maxmempool: i32,
    pub maxorphantx: i32,
}

#[derive(Debug, Deserialize)]
pub struct SpamProtection {
    pub limitfreerelay: i32,
    pub minrelaytxfee: f32,
}

#[derive(Debug, Deserialize)]
pub struct Connections {
    pub maxconnections: i32,
}

#[derive(Debug, Deserialize)]
pub struct Civkitd {
    pub network: String,
    pub noise_port: i32,
    pub nostr_port: i32,
    pub cli_port: i32,
}

