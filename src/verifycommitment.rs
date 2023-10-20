use bitcoin_hashes::{sha256, Hash};
use crate::inclusionproof::{InclusionProof};
use rs_merkle::{MerkleTree, MerkleProof};
use rs_merkle::algorithms::Sha256;

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
