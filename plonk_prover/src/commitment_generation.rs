#![cfg(feature = "proof_generator")]

use rand::RngCore;
use rand_core::OsRng;
use shared::functions::scalar_to_bytes;
use shared::public_types::*;

/// Generate randomness, nullifier, commitment and nullifier hash
pub fn generate_commitment() -> GeneratedCommitment {
    // Use OsRng as CSPRNG
    let mut os_rng = OsRng::default();

    // Generate nullifier and randomness
    let nullifier = os_rng.next_u32();
    let randomness = os_rng.next_u32();

    // Compute commitment
    let commitment =
        dusk_poseidon::sponge::hash(&[(nullifier as u64).into(), (randomness as u64).into()]);

    // Convert commitment to bytes
    let commitment_bytes = scalar_to_bytes(commitment);

    // Compute nullifier hash
    let nullifier_hash = dusk_poseidon::sponge::hash(&[(nullifier as u64).into()]);

    // Convert nullifier hash to bytes
    let nullifier_hash_bytes = scalar_to_bytes(nullifier_hash);

    GeneratedCommitment {
        nullifier,
        randomness,
        commitment_bytes,
        nullifier_hash_bytes,
    }
}

pub struct GeneratedCommitment {
    pub nullifier: u32,
    pub randomness: u32,
    pub commitment_bytes: PoseidonHash,
    pub nullifier_hash_bytes: PoseidonHash,
}
