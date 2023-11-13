use crate::util;
use std::fs;
use crate::config::Config;
use crate::inclusionproof::InclusionProof;
use crate::verifycommitment::{verify_merkle_root_inclusion};
use bitcoincore_rpc::bitcoin::Txid;
use bitcoin::BlockHash;
use bitcoincore_rpc::Client;
use bitcoincore_rpc::json::{GetRawTransactionResult};
use serde_json::from_str;

const tx_data: &str = r#"
{
    "txid": "b891111d35ffc72709140b7bd2a82fde20deca53831f42a96704dede42c793d2",
    "hash": "b891111d35ffc72709140b7bd2a82fde20deca53831f42a96704dede42c793d2",
    "version": 2,
    "size": 194,
    "vsize": 194,
    "weight": 776,
    "locktime": 0,
    "vin": [
      {
        "txid": "047352f01e5e3f8adc04a797311dde3917f274e55ceafb78edc39ff5d87d16c5",
        "vout": 0,
        "scriptSig": {
          "asm": "0 30440220049d3138f841b63e96725cb9e86a53a92cd1d9e1b0740f5d4cd2ae0bcab684bf0220208d555c7e24e4c01cf67dfa9161091533e9efd6d1602bb53a49f7195c16b037[ALL] 5121036bd7943325ed9c9e1a44d98a8b5759c4bf4807df4312810ed5fc09dfb967811951ae",
          "hex": "004730440220049d3138f841b63e96725cb9e86a53a92cd1d9e1b0740f5d4cd2ae0bcab684bf0220208d555c7e24e4c01cf67dfa9161091533e9efd6d1602bb53a49f7195c16b03701255121036bd7943325ed9c9e1a44d98a8b5759c4bf4807df4312810ed5fc09dfb967811951ae"
        },
        "sequence": 4294967293
      }
    ],
    "vout": [
      {
        "value": 0.01040868,
        "n": 0,
        "scriptPubKey": {
          "asm": "OP_HASH160 29d13058087ddf2d48de404376fdcb5c4abff4bc OP_EQUAL",
          "desc": "addr(35W8E71bdDhQw4ZC7uUZvXG3qhyWVYxfMB)#4rtfrxzg",
          "hex": "a91429d13058087ddf2d48de404376fdcb5c4abff4bc87",
          "address": "35W8E71bdDhQw4ZC7uUZvXG3qhyWVYxfMB",
          "type": "scripthash"
        }
      }
    ],
    "hex": "0200000001c5167dd8f59fc3ed78fbea5ce574f21739de1d3197a704dc8a3f5e1ef0527304000000006f004730440220049d3138f841b63e96725cb9e86a53a92cd1d9e1b0740f5d4cd2ae0bcab684bf0220208d555c7e24e4c01cf67dfa9161091533e9efd6d1602bb53a49f7195c16b03701255121036bd7943325ed9c9e1a44d98a8b5759c4bf4807df4312810ed5fc09dfb967811951aefdffffff01e4e10f000000000017a91429d13058087ddf2d48de404376fdcb5c4abff4bc8700000000","blockhash":"000000000000000000036cb20420528cf0f00abb3a5716d80b5c87146b764d47",
    "confirmations":15235,
    "time":1690540748,
    "blocktime":1690540748
}"#;
pub const test_merkle_root: &str = "8d0ad2782d8f6e3f63c6f9611841c239630b55061d558abcc6bac53349edac70";

pub struct MockClient {}

impl MockClient {
    pub fn new() -> Self {
      MockClient {}
    }
    pub fn get_raw_transaction_info(&self, txid: &Txid, blockhash: Option<&BlockHash>) -> Result<GetRawTransactionResult, Box<dyn std::error::Error>> {
      let tx_info: GetRawTransactionResult = from_str(tx_data)?;
      Ok(tx_info)
    }
}

#[test]
fn test_verify_merkle_root_inclusion() {

    let data_dir = util::get_default_data_dir();

	  let config_path = data_dir.join("example-config.toml");

    // Read the configuration file
    let contents = fs::read_to_string(&config_path);
    let config = match contents {
        Ok(data) => {
            toml::from_str(&data).expect("Could not deserialize the config file content")
        },
        Err(_) => {
            // If there's an error reading the file, use the default configuration
            Config::default()
        }
    };

    let mut inclusion_proof = InclusionProof::new(
        "".to_string(), 
        "".to_string(), 
        "".to_string(), 
        Vec::new(), 
        config.clone()
    );

    let result = verify_merkle_root_inclusion("b891111d35ffc72709140b7bd2a82fde20deca53831f42a96704dede42c793d2".to_string(), &mut inclusion_proof);
    assert_eq!(result, true);
}