use crate::util;
use std::fs;
use crate::config::Config;
use crate::inclusionproof::InclusionProof;
use crate::verifycommitment::{verify_merkle_root_inclusion};
use bitcoin::BlockHash;
use crate::rpcclient::Client;
use serde_json::{from_str, Value};

const TX_DATA: &str = r#"
{
  "txid": "77a11c98ee0d323eca29466bc962325af0f3c2b73b238007030daa3e6f62837a",
  "hash": "26c82876b561f73f75f327423df10fac22d0254c1be751d98f369fe117ad14f6",
  "version": 1,
  "size": 192,
  "vsize": 110,
  "weight": 438,
  "locktime": 0,
  "vin": [
    {
      "txid": "4b9b32dfae050222b4c07f1c730b2611d6e12750c961e700a5e94febdff6cbb0",
      "vout": 0,
      "scriptSig": {
        "asm": "",
        "hex": ""
      },
      "txinwitness": [
        "3045022100a3ccdae300fd23ba8cf50000b524d082b91f8c2b65430eddcad945a5252683fd022007bed3f867908d209647b057afa61ca401e1c230903936bdd0769cd80a88da3001",
        "0220a35481d8e9ddab95382bca286a3ddc73e5349f670321215f5be8669cfcf3ca"
      ],
      "sequence": 4294967293
    }
  ],
  "vout": [
    {
      "value": 0.0009868,
      "n": 0,
      "scriptPubKey": {
        "asm": "0 996d5b0d4ebeba02e72205c9731c8f6ab9b5f7b6",
        "desc": "addr(tb1qn9k4kr2wh6aq9eezqhyhx8y0d2umtaakd08nzl)#yrwe0736",
        "hex": "0014996d5b0d4ebeba02e72205c9731c8f6ab9b5f7b6",
        "address": "tb1qn9k4kr2wh6aq9eezqhyhx8y0d2umtaakd08nzl",
        "type": "witness_v0_keyhash"
      }
    }
  ],
  "hex": "01000000000101b0cbf6dfeb4fe9a500e761c95027e1d611260b731c7fc0b4220205aedf329b4b0000000000fdffffff017881010000000000160014996d5b0d4ebeba02e72205c9731c8f6ab9b5f7b602483045022100a3ccdae300fd23ba8cf50000b524d082b91f8c2b65430eddcad945a5252683fd022007bed3f867908d209647b057afa61ca401e1c230903936bdd0769cd80a88da3001210220a35481d8e9ddab95382bca286a3ddc73e5349f670321215f5be8669cfcf3ca00000000",
  "blockhash": "000000000000c68a420a4c6e319555ed49e1d24dc319f66ce82865e44f020249",
  "confirmations": 5,
  "time": 1708703756,
  "blocktime": 1708703756
}"#;

pub const TEST_TXID: &str = "77a11c98ee0d323eca29466bc962325af0f3c2b73b238007030daa3e6f62837a";
pub const TEST_MERKLE_ROOT: &str = "9a0ccfb086b7469053a2584a3d57e92d7dd004e24c49ff59d9b1fe6cbd656495";
pub const TEST_COMMITMENT: &str = "4823a86fb00a38d36d1e93f6456ff61fed70b58ee3e08cf84e7980608c41ca53";

#[test]
fn test_verify_merkle_root_inclusion() {

    let json_value: Value = from_str(TX_DATA).unwrap();
    let mut inclusion_proof = InclusionProof::new(
        TEST_TXID.to_string(), 
        TEST_COMMITMENT.to_string(), 
        TEST_MERKLE_ROOT.to_string(),
        Vec::new(),
        "".to_string(),
        json_value,
        Config::default(),
    );

    let result = verify_merkle_root_inclusion(&mut inclusion_proof);
    assert_eq!(result, true);
}
