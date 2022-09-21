#![cfg(feature = "js")]

use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::array::TryFromSliceError;

use wasm_bindgen::prelude::wasm_bindgen;

///Depth which is used in Slushie mixer contract
use shared::constants::DEFAULT_DEPTH;
use shared::public_types::*;

use crate::commitment_generation::{generate_commitment as commitment_gen, GeneratedCommitment};
use crate::hasher::Poseidon;
use crate::merkle_tree::{MerkleTree, MerkleTreeError};
use crate::proof_generation::prove;

#[cfg(feature = "js_include_pd")]
use crate::proof_generation::prove_with_vd;

const SERIALIZED_PUBLIC_PARAMETERS: &[u8] = include_bytes!("../../public-parameters/pp-test");

#[cfg(feature = "js_include_pd")]
const PROVER_DATA: &[u8] = include_bytes!("../../public-parameters/pd-test");

#[cfg(feature = "js_include_pd")]
const COMMIT_KEY: &[u8] = include_bytes!("../../public-parameters/commit-key-test");

///Generate serialized proof which is compatible with js and can be used in frontend
#[allow(clippy::too_many_arguments)]
#[allow(non_snake_case)]
#[wasm_bindgen]
pub fn generate_proof(
    l: usize,
    R: &[u8],
    o: &[u8],
    k: u32,
    r: u32,
    A: &[u8],
    t: &[u8],
    f: u64,
) -> Result<Vec<u8>, js_sys::Error> {
    //Read opening from bytes array
    let mut opening = [[0; 32]; DEFAULT_DEPTH];
    for i in 0..DEFAULT_DEPTH {
        for j in 0..32 {
            opening[i][j] = o[i * 32 + j];
        }
    }

    // Parse arguments
    let R: PoseidonHash = R
        .try_into()
        .map_err(|err: TryFromSliceError| js_sys::Error::new(&err.to_string()))?;
    let A = A
        .try_into()
        .map_err(|err: TryFromSliceError| js_sys::Error::new(&err.to_string()))?;
    let t = t
        .try_into()
        .map_err(|err: TryFromSliceError| js_sys::Error::new(&err.to_string()))?;

    //Generate proof
    prove(SERIALIZED_PUBLIC_PARAMETERS, l, R, opening, k, r, A, t, f)
        .map_err(|err| js_sys::Error::new(&format!("{:?}", err)))
        .map(|proof| proof.into_iter().collect())
}

///Generate serialized proof which is compatible with js and can be used in frontend
#[cfg(feature = "js_include_pd")]
#[allow(clippy::too_many_arguments)]
#[allow(non_snake_case)]
#[wasm_bindgen]
pub fn generate_proof_with_pd(
    l: usize,
    R: &[u8],
    o: &[u8],
    k: u32,
    r: u32,
    A: &[u8],
    t: &[u8],
    f: u64,
) -> Result<Vec<u8>, js_sys::Error> {
    //Read opening from bytes array
    let mut opening = [[0; 32]; DEFAULT_DEPTH];
    for i in 0..DEFAULT_DEPTH {
        for j in 0..32 {
            opening[i][j] = o[i * 32 + j];
        }
    }

    // Parse arguments
    let R: PoseidonHash = R
        .try_into()
        .map_err(|err: TryFromSliceError| js_sys::Error::new(&err.to_string()))?;
    let A = A
        .try_into()
        .map_err(|err: TryFromSliceError| js_sys::Error::new(&err.to_string()))?;
    let t = t
        .try_into()
        .map_err(|err: TryFromSliceError| js_sys::Error::new(&err.to_string()))?;

    //Generate proof
    prove_with_vd(PROVER_DATA, COMMIT_KEY, l, R, opening, k, r, A, t, f)
        .map_err(|err| js_sys::Error::new(&format!("{:?}", err)))
        .map(|proof| proof.into_iter().collect())
}

/// Generate randomness, nullifier, commitment and nullifier hash
#[wasm_bindgen]
pub fn generate_commitment() -> js_sys::Array {
    let GeneratedCommitment {
        nullifier,
        randomness,
        commitment_bytes,
        nullifier_hash_bytes,
    } = commitment_gen();

    // Set nullifier as js number
    let js_nullifier = js_sys::Number::from(nullifier);

    // Set randomness as js number
    let js_randomness = js_sys::Number::from(randomness);

    // Set commitment as js Uint8Array
    let js_commitment = js_sys::Uint8Array::new_with_length(32);
    js_commitment.copy_from(&commitment_bytes);

    // Set nullifier hash as js Uint8Array
    let js_nullifier_hash = js_sys::Uint8Array::new_with_length(32);
    js_nullifier_hash.copy_from(&nullifier_hash_bytes);

    js_sys::Array::of4(
        &js_nullifier,
        &js_randomness,
        &js_commitment,
        &js_nullifier_hash,
    )
}

/// Generate tree opening and path
#[wasm_bindgen]
pub fn generate_tree_opening(
    flat_commitments: &[u8],
    leaf_index: usize,
) -> Result<js_sys::Array, js_sys::Error> {
    //Read commitments from bytes array
    let mut commitments = vec![[0; 32]; flat_commitments.len() / 32];
    for i in 0..commitments.len() {
        for j in 0..32 {
            commitments[i][j] = flat_commitments[i * 32 + j];
        }
    }

    //Create merkle tree
    let tree: MerkleTree<DEFAULT_DEPTH, Poseidon> = commitments
        .as_slice()
        .try_into()
        .map_err(|err: MerkleTreeError| js_sys::Error::new(&format!("{:?}", err)))?;

    // Get tree opening for leaf index
    let tree_opening = tree
        .get_opening(leaf_index)
        .map_err(|err: MerkleTreeError| js_sys::Error::new(&format!("{:?}", err)))?;

    // Get tree path for leaf index
    let tree_path = tree
        .get_path(leaf_index)
        .map_err(|err: MerkleTreeError| js_sys::Error::new(&format!("{:?}", err)))?;

    // Convert to js types
    let js_opening = js_sys::Uint8Array::new_with_length(32 * DEFAULT_DEPTH as u32);
    js_opening.copy_from(&tree_opening.into_iter().flatten().collect::<Vec<u8>>());

    let js_path = js_sys::Uint8Array::new_with_length(DEFAULT_DEPTH as u32);
    js_path.copy_from(&tree_path);

    Ok(js_sys::Array::of2(&js_opening, &js_path))
}

/// To run wasm tests:
/// wasm-pack test --node -r --features js
#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use dusk_poseidon::sponge;
    use wasm_bindgen_test::*;

    ///Depth which is used in Slushie mixer contract
    use shared::constants::DEFAULT_DEPTH;
    use shared::functions::*;

    use crate::proof_generation::prove_with_vd;
    use crate::proof_verification::verify_with_vd;
    use crate::tests::*;

    ///Test for checking circuit works in wasm
    #[wasm_bindgen_test]
    fn generate_proof_test() {
        const DEPTH: usize = DEFAULT_DEPTH;
        let k = rand::random::<u32>();
        let r = rand::random::<u32>();
        let l = rand::random::<u16>() as usize;
        let f = rand::random::<u64>();

        let (R, o) = setup::<DEPTH>(k, r, l);

        let proof = &prove_with_vd(PD, COMMIT_KEY, l, R, o, k, r, PAYOUT, RELAYER, f).unwrap();

        let h = sponge::hash(&[(k as u64).into()]);

        verify_with_vd(
            VD,
            OPENING_KEY,
            scalar_to_bytes(h),
            R,
            PAYOUT,
            RELAYER,
            f,
            proof,
        )
        .unwrap();
    }
}
