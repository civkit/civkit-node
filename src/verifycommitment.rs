use bitcoin_hashes::{sha256, Hash, hash160};
use crate::inclusionproof::{InclusionProof};
use rs_merkle::{MerkleTree, MerkleProof};
use rs_merkle::algorithms::Sha256;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use bitcoincore_rpc::bitcoin::Txid;
use std::str::FromStr;
use hex::{encode, decode};
use bip32::{ExtendedPublicKey, ExtendedKeyAttrs, PublicKey, DerivationPath, ChildNumber};

pub fn verify_commitments(event_commitments: Vec<Vec<u8>>, latest_commitment: Vec<u8>) -> bool {
    let mut concatenated_hash = Vec::new();

    for event_commitment in &event_commitments {
        if concatenated_hash.is_empty() {
            concatenated_hash.extend_from_slice(&event_commitments[0]);
        } else {
            concatenated_hash.extend_from_slice(&event_commitment);
            concatenated_hash = sha256::Hash::hash(&concatenated_hash).to_vec();
        }
    }

    let calculated_commitment = concatenated_hash;

    calculated_commitment == latest_commitment
}

pub fn verify_slot_proof(slot: usize, inclusion_proof: &mut InclusionProof) -> bool {
    let merkle_root = inclusion_proof.merkle_root.lock().unwrap();
    let commitment = inclusion_proof.commitment.lock().unwrap();
    let ops = inclusion_proof.ops.lock().unwrap();
    let ops_commitments: Vec<&str> = ops.iter().map(|pth| pth.commitment.as_str()).collect();

    let leaf_hashes: Vec<[u8; 32]> = ops_commitments
        .iter()
        .map(|x| sha256::Hash::hash(x.as_bytes()).into_inner())
        .collect();
    
    let leaf_to_prove = leaf_hashes.get(slot).unwrap();

    let merkle_tree = MerkleTree::<Sha256>::from_leaves(&leaf_hashes);
    let merkle_proof = merkle_tree.proof(&[slot]);
    let merkle_root = merkle_tree.root().unwrap();

    let proof_bytes = merkle_proof.to_bytes();

    let proof = MerkleProof::<Sha256>::try_from(proof_bytes).unwrap();

    return proof.verify(merkle_root, &[slot], &[*leaf_to_prove], leaf_hashes.len());
}

pub fn verify_merkle_root_inclusion(txid: String, inclusion_proof: &mut InclusionProof) -> bool {
    let client = Client::new(format!("{}:{}/", inclusion_proof.config.bitcoind_params.host, inclusion_proof.config.bitcoind_params.port).as_str(),
                        Auth::UserPass(inclusion_proof.config.bitcoind_params.rpc_user.to_string(),
                                        inclusion_proof.config.bitcoind_params.rpc_password.to_string())).unwrap();
    
    match client.get_raw_transaction_info(&Txid::from_str(&txid).unwrap(), None) {
        Ok(transaction) => {
            let script_pubkey_from_tx = encode(&transaction.vout[0].script_pub_key.hex);
            let merkle_root = inclusion_proof.merkle_root.lock().unwrap().as_bytes().to_vec();
            let rev_merkle_root: Vec<u8> = merkle_root.iter().rev().cloned().collect();
            let rev_merkle_root_hex = encode(rev_merkle_root);

            let path = get_path_from_commitment(rev_merkle_root_hex).unwrap();

            let initial_public_key_hex = &inclusion_proof.config.mainstay.base_pubkey;
            let initial_chain_code_hex = &inclusion_proof.config.mainstay.chain_code;

            let initial_public_key_bytes = decode(initial_public_key_hex).expect("Invalid public key hex string");
            let mut public_key_bytes = [0u8; 33];
            public_key_bytes.copy_from_slice(&initial_public_key_bytes);

            let initial_public_key = bip32::secp256k1::PublicKey::from_bytes(public_key_bytes).expect("Invalid public key");
            let mut initial_chain_code = decode(initial_chain_code_hex).expect("Invalid chain code hex string");
            let mut initial_chain_code_array = [0u8; 32];
            initial_chain_code_array.copy_from_slice(initial_chain_code.as_mut_slice());

            let attrs = ExtendedKeyAttrs {
                depth: 0,
                parent_fingerprint: Default::default(),
                child_number: Default::default(),
                chain_code: initial_chain_code_array,
            };

            let initial_extended_pubkey = ExtendedPublicKey::new(initial_public_key, attrs);
            let (child_pubkey, child_chain_code) = derive_child_key_and_chaincode(&initial_extended_pubkey, &path.to_string());
            
            let script = create_1_of_1_multisig_script(child_pubkey);

            let address = bitcoin::Address::p2sh(&script, bitcoin::Network::Bitcoin).unwrap();
            let script_pubkey = encode(address.script_pubkey());
                                            
            return script_pubkey == script_pubkey_from_tx;
        }
        Err(error) => {
            println!("Error: {:?}", error);
        }
    }
    return false;
}

pub fn get_path_from_commitment(commitment: String) -> Option<String> {
    let path_size = 16;
    let child_size = 4;

    if commitment.len() != path_size * child_size {
        return None;
    }

    let mut derivation_path = String::new();
    for it in 0..path_size {
        let index = &commitment[it * child_size..it * child_size + child_size];
        let decoded_index = u64::from_str_radix(index, 16).unwrap();
        derivation_path.push_str(&decoded_index.to_string());
        if it < path_size - 1 {
            derivation_path.push('/');
        }
    }

    Some(derivation_path)
}

fn derive_child_key_and_chaincode(mut parent: &ExtendedPublicKey<bip32::secp256k1::PublicKey>, path: &str) -> (bip32::secp256k1::PublicKey, [u8; 32]) {
    let mut extended_key = parent.clone();
    let mut chain_code = parent.attrs().chain_code.clone();
    let mut public_key = parent.public_key().clone();
    for step in path.split('/') {
        match step {
            "m" => continue,
            number => {
                if let Ok(index) = number.parse::<u32>() {
                    let new_extended_key = extended_key.derive_child(ChildNumber(index)).expect("Failed to derive child key");
                    chain_code = new_extended_key.attrs().chain_code;
                    public_key = *new_extended_key.public_key();
                    extended_key = new_extended_key.clone();
                } else {
                    panic!("Invalid derivation path step: {}", step);
                }
            }
        }
    }
    (public_key, chain_code)
}

fn create_1_of_1_multisig_script(pubkey: bip32::secp256k1::PublicKey) -> bitcoin::blockdata::script::Script {
    let public_key = bitcoin::util::key::PublicKey {
        inner: bitcoin::secp256k1::PublicKey::from_slice(&pubkey.to_bytes()).unwrap(),
        compressed: true,
    };
    let script = bitcoin::blockdata::script::Builder::new()
        .push_opcode(bitcoin::blockdata::opcodes::all::OP_PUSHNUM_1)
        .push_key(&public_key)
        .push_opcode(bitcoin::blockdata::opcodes::all::OP_PUSHNUM_1)
        .push_opcode(bitcoin::blockdata::opcodes::all::OP_CHECKMULTISIG)
        .into_script();
    script
}