use bitcoin_hashes::{sha256, Hash};

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
