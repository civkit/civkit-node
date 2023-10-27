use bitcoin_hashes::{sha256, Hash};
use crate::inclusionproof::{InclusionProof};
use rs_merkle::{MerkleTree, MerkleProof};
use rs_merkle::algorithms::Sha256;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use bitcoincore_rpc::bitcoin::Txid;
use crate::config::Config;
use std::str::FromStr;
use hex::{encode, decode};
use bip32::{ExtendedPublicKey, ExtendedKeyAttrs, PublicKey};

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
    let client = Client::new(format!("{}:{}/", inclusion_proof.config.bitcoind_client.host, inclusion_proof.config.bitcoind_client.port).as_str(),
                        Auth::UserPass(inclusion_proof.config.bitcoind_client.rpc_user.to_string(),
                                        inclusion_proof.config.bitcoind_client.rpc_password.to_string())).unwrap();
    
    match client.get_raw_transaction_info(&Txid::from_str(&txid).unwrap(), None) {
        Ok(transaction) => {
            let script_addr = &transaction.vout[0].script_pub_key.hex;
            let commitment = inclusion_proof.merkle_root.lock().unwrap().as_bytes().to_vec();
            let commitment_path = get_path_from_commitment(commitment);

            let initial_public_key_hex = &inclusion_proof.config.mainstay.base_pubkey;
            let initial_chain_code_hex = &inclusion_proof.config.mainstay.chain_code;

            let initial_public_key_bytes = decode(initial_public_key_hex).expect("Invalid public key hex string");
            let mut public_key_bytes = [0u8; 33];
            public_key_bytes.copy_from_slice(&initial_public_key_bytes);
            let initial_public_key: PublicKey = PublicKey::from_bytes(public_key_bytes).expect("Invalid public key");
            let initial_chain_code = decode(initial_chain_code_hex).expect("Invalid chain code hex string");
            let mut initial_chain_code_array = [0u8; 32];
            initial_chain_code_array.copy_from_slice(initial_chain_code.as_mut_slice());

            let initial_extended_pubkey = ExtendedPublicKey {
                public_key: initial_public_key,
                attrs: ExtendedKeyAttrs {
                    depth: 0,
                    parent_fingerprint: Default::default(),
                    child_number: Default::default(),
                    chain_code: initial_chain_code_array,
                },
            };
        
        }
        Err(error) => {
            println!("Error: {:?}", error);
        }
    }
                                    
    return true;
}

pub fn get_path_from_commitment(commitment: Vec<u8>) -> Option<Vec<u8>> {
    let path_size = 16;
    let child_size = 2;

    if commitment.len() != path_size * child_size {
        return None;
    }

    let mut derivation_path = Vec::new();
    for it in 0..path_size {
        let index = &commitment[it * child_size..it * child_size + child_size];
        derivation_path.push(index.iter().cloned().next().unwrap());
    }

    Some(derivation_path)
}
