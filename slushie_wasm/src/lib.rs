#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;

///Depth which is used in Slushie mixer contract
use shared::constants::DEFAULT_DEPTH;

use plonk_prover::*;

use core::array::TryFromSliceError;
use wasm_bindgen::prelude::wasm_bindgen;

///Generation serialized proof which is compatible with js and can be used in frontend
#[allow(clippy::too_many_arguments)]
#[allow(non_snake_case)]
#[wasm_bindgen]
pub fn generate_proof(
    pp: &[u8],
    l: usize,
    R: &[u8],
    o: &[u8],
    k: u32,
    r: u32,
    A: &[u8],
    t: &[u8],
    f: u64,
) -> Result<Vec<u8>, js_sys::Error> {
    //Reading opening from bytes array
    let mut opening = [[0; 32]; DEFAULT_DEPTH];
    for i in 0..DEFAULT_DEPTH {
        for j in 0..32 {
            opening[i][j] = o[i * 32 + j];
        }
    }

    // Parsing arguments
    let R: PoseidonHash = R
        .try_into()
        .map_err(|err: TryFromSliceError| js_sys::Error::new(&err.to_string()))?;
    let A: PoseidonHash = A
        .try_into()
        .map_err(|err: TryFromSliceError| js_sys::Error::new(&err.to_string()))?;
    let t = t
        .try_into()
        .map_err(|err: TryFromSliceError| js_sys::Error::new(&err.to_string()))?;

    //Generate proof
    prove(pp, l, R, opening, k, r, A, t, f)
        .map_err(|err| js_sys::Error::new(&format!("{:?}", err)))
        .map(|proof| proof.into_iter().collect())
}

/// To run wasm tests:
/// wasm-pack test --node -r
#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use dusk_plonk::prelude::*;
    use dusk_poseidon::sponge;
    use hex_literal::hex;
    use plonk_prover::index_to_path;
    use plonk_prover::verify;
    use plonk_prover::PoseidonHash;
    use plonk_prover::Pubkey;
    use shared::functions::scalar_to_bytes;
    use shared::functions::u64_to_bytes;
    use wasm_bindgen_test::*;

    ///Setup function for every test
    fn setup<const DEPTH: usize>(
        k: u32,
        r: u32,
        l: usize,
    ) -> (PublicParameters, PoseidonHash, [PoseidonHash; DEPTH]) {
        //Setup public parameters from file to reduce tests time
        let pp = PublicParameters::from_slice(include_bytes!("../pp-test")).unwrap();

        //Calculate commitment
        let commitment = sponge::hash(&[(k as u64).into(), (r as u64).into()]);

        //Calculate opening
        let (R, o) = get_opening(l, commitment);

        // Return public parameters, verifier data, root hash and opening
        (pp, R, o)
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

        let (pp, R, o) = setup::<DEPTH>(k, r, l);

        let proof = &generate_proof(
            &pp.to_var_bytes(),
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

        verify::<DEPTH>(
            &pp.to_var_bytes(),
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
