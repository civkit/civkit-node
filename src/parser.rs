use std::fs;
use toml;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    performance: Performance,
    spam_protection: SpamProtection,
    connections: Connections,
    civkitd: Civkitd,
}

#[derive(Debug, Deserialize)]
struct Performance {
    par: i32,
    dbcache: i32,
    blocksonly: i32,
    maxuploadtarget: i32,
    mempoolexpiry: i32,
    maxmempool: i32,
    maxorphantx: i32,
}

#[derive(Debug, Deserialize)]
struct SpamProtection {
    limitfreerelay: i32,
    minrelaytxfee: f32,
}

#[derive(Debug, Deserialize)]
struct Connections {
    maxconnections: i32,
}

#[derive(Debug, Deserialize)]
struct Civkitd {
    network: String,
    noise_port: i32,
    nostr_port: i32,
    cli_port: i32,
}

fn main() {
    let contents = fs::read_to_string("./config.toml")
        .expect("Something went wrong reading the file");

    let config: Config = toml::from_str(&contents)
        .expect("Could not deserialize the config file");

    println!("{:#?}", config);
}
