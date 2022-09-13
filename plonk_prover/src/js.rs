#![cfg(feature = "js")]

use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::hasher::Poseidon;
use crate::merkle_tree::{MerkleTree, MerkleTreeError};

///Depth which is used in Slushie mixer contract
use shared::constants::DEFAULT_DEPTH;
use shared::public_types::*;

use crate::commitment_generation::{generate_commitment as commitment_gen, GeneratedCommitment};
use crate::proof_generation::prove;

use core::array::TryFromSliceError;
use wasm_bindgen::prelude::wasm_bindgen;

const SERIALIZED_PUBLIC_PARAMETERS: &[u8] = include_bytes!("../../public-parameters/pp-test");

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
    use crate::proof_verification::verify_with_vd;
    use crate::utils::index_to_path;
    use alloc::vec::Vec;
    use dusk_bytes::Serializable;
    use dusk_plonk::prelude::*;
    use dusk_poseidon::sponge;
    use hex_literal::hex;
    use shared::functions::*;
    use shared::public_types::*;
    use wasm_bindgen_test::*;

    ///Setup function for every test
    fn setup<'a, const DEPTH: usize>(
        k: u32,
        r: u32,
        l: usize,
    ) -> (
        &'a [u8],
        &'a [u8; OpeningKey::SIZE],
        PoseidonHash,
        [PoseidonHash; DEPTH],
    ) {
        // Setup verifier data and opening key from file
        let vd = include_bytes!("../../public-parameters/vd-test");
        let opening_key = include_bytes!("../../public-parameters/op-key-test");

        //Calculate commitment
        let commitment = sponge::hash(&[(k as u64).into(), (r as u64).into()]);

        //Calculate opening
        let (R, o) = get_opening(l, commitment);

        // Return public parameters, verifier data, root hash and opening
        (vd, opening_key, R, o)
    }

    ///Get random opening and root for its
    fn get_opening<const DEPTH: usize>(
        commitment_index: usize,
        commitment: BlsScalar,
    ) -> (PoseidonHash, [PoseidonHash; DEPTH]) {
        let mut opening = [[0; 32]; DEPTH];
        //Path from index
        let path = index_to_path::<DEPTH>(commitment_index).expect("WrongIndex");
        let mut last_hash = BlsScalar::zero();

        for i in 0..DEPTH {
            //Last calculated hash
            let current_hash = if i == 0 { commitment } else { last_hash };
            //Random sisters hash
            let sister_hash = BlsScalar::from(rand::random::<u64>());
            opening[i] = u64_to_bytes(sister_hash.0);

            //Calculate next hash using path and sisters hash
            let left = if path[i] == 1 {
                sister_hash
            } else {
                current_hash
            };
            let right = if path[i] == 1 {
                current_hash
            } else {
                sister_hash
            };

            last_hash = sponge::hash(&[left, right]);
        }

        //Return root and opening
        (u64_to_bytes(last_hash.0), opening)
    }

    ///Set constant PAYOUT and RELAYER address
    const PAYOUT: Pubkey = hex!("38c4c4c0f0e9de905b304b60f3ab77b47e2f6b4a388b7859373c6e6a1581708a");
    const RELAYER: Pubkey =
        hex!("92fba99dfb7832c4268e299efb9cd3aaad7153bbee9974729340b528d276936e");

    ///Depth which is used in Slushie mixer contract
    use shared::constants::DEFAULT_DEPTH;

    use crate::generate_proof;

    ///Test for checking circuit works in wasm
    #[wasm_bindgen_test]
    fn generate_proof_test() {
        const DEPTH: usize = DEFAULT_DEPTH;
        let k = rand::random::<u32>();
        let r = rand::random::<u32>();
        let l = rand::random::<u16>() as usize;
        let f = rand::random::<u64>();

        let (vd, opening_key, R, o) = setup::<DEPTH>(k, r, l);

        let proof = &generate_proof(
            l,
            &R,
            &o.into_iter().flatten().collect::<Vec<u8>>()[..],
            k,
            r,
            &PAYOUT,
            &RELAYER,
            f,
        )
        .unwrap()
        .try_into()
        .unwrap();

        let h = sponge::hash(&[(k as u64).into()]);

        verify_with_vd(
            vd,
            opening_key,
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
