#![cfg_attr(not(feature = "std"), no_std)]

mod circuit;
mod utils;

use circuit::*;
use shared::functions::bytes_to_u64;
use utils::*;

use dusk_bytes::Serializable;
use dusk_plonk::prelude::*;
use dusk_poseidon::sponge;
use rand_core::OsRng;

#[macro_use]
extern crate alloc;

#[cfg(target_arch = "wasm32")]
use alloc::vec::Vec;

#[cfg(target_arch = "wasm32")]
use alloc::string::ToString;

#[cfg(target_arch = "wasm32")]
use alloc::format;

///Depth which is used in Slushie mixer contract
#[cfg(target_arch = "wasm32")]
use shared::constants::DEFAULT_DEPTH;

///Constant which should be equal during generating proof and verifying its
const TRANSCRIPT_INIT: &[u8; 7] = b"slushie";

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(target_arch = "wasm32")]
use core::array::TryFromSliceError;

///Generation serialized proof which is compatible with js and can be used in frontend
#[cfg(target_arch = "wasm32")]
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
    let mut opening = [[0; 32]; DEFAULT_DEPTH];

    for i in 0..DEFAULT_DEPTH {
        for j in 0..32 {
            opening[i][j] = o[i * 32 + j];
        }
    }

    let R: PoseidonHash = R
        .try_into()
        .map_err(|err: TryFromSliceError| js_sys::Error::new(&err.to_string()))?;
    let A: PoseidonHash = A
        .try_into()
        .map_err(|err: TryFromSliceError| js_sys::Error::new(&err.to_string()))?;
    let t = t
        .try_into()
        .map_err(|err: TryFromSliceError| js_sys::Error::new(&err.to_string()))?;

    prove(pp, l, R, opening, k, r, A, t, f)
        .map_err(|err| js_sys::Error::new(&format!("{:?}", err)))
        .map(|proof| proof.into_iter().collect())
}

///Generation serialized proof
#[allow(clippy::too_many_arguments)]
#[allow(non_snake_case)]
pub fn prove<const DEPTH: usize>(
    //Public parameters
    pp: &[u8],
    //Leaf index
    l: usize,
    //Root
    R: PoseidonHash,
    //Tree opening
    o: [PoseidonHash; DEPTH],
    //Nullifier
    k: u32,
    //Randomness
    r: u32,
    //Recipient address
    A: Pubkey,
    //Relayer address
    t: Pubkey,
    //Fee
    f: u64,
) -> Result<[u8; 1040], Error> {
    //Read public parameters
    let pp = PublicParameters::from_slice(pp)?;

    //Compile circuit
    let mut circuit = SlushieCircuit::<DEPTH>::default();
    let (pk, _vd) = circuit.compile(&pp)?;

    //Create circuit
    let mut circuit = SlushieCircuit::<DEPTH> {
        R: BlsScalar(bytes_to_u64(R)),
        r: (r as u64).into(),
        k: (k as u64).into(),
        h: sponge::hash(&[(k as u64).into()]),
        A: BlsScalar::from_raw(bytes_to_u64(A)),
        t: BlsScalar::from_raw(bytes_to_u64(t)),
        f: f.into(),
        o: Array(o),
        p: Array(index_to_path(l).map_err(|_| Error::ProofVerificationError)?),
    };

    //Generate proof
    circuit
        .prove(&pp, &pk, TRANSCRIPT_INIT, &mut OsRng)
        .map(|proof| proof.to_bytes())
}

/// Tests take some time due to proof generating. Recommend running them in release mode with parallel feature
/// cargo test -r --features parallel  
/// To run wasm tests:
/// wasm-pack test --node -r
#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;
    use dusk_plonk::prelude::*;
    use dusk_poseidon::sponge;
    use hex_literal::hex;
    use shared::functions::u64_to_bytes;

    use super::*;

    const MAX_DEGREE: usize = CIRCUIT_SIZE;

    /// Test for this situation:
    ///         R
    ///        / \
    ///       n   o[1]
    ///      / \
    ///   o[0]  hash(k || r)
    ///    0        1
    #[test]
    fn verification_success() {
        //Set depth
        const DEPTH: usize = 2;

        //Set input examples
        const PAYOUT: Pubkey =
            hex!("38c4c4c0f0e9de905b304b60f3ab77b47e2f6b4a388b7859373c6e6a1581708a");
        const RELAYER: Pubkey =
            hex!("92fba99dfb7832c4268e299efb9cd3aaad7153bbee9974729340b528d276936e");
        let k = 3141592653;
        let r = 1;
        let f = 0;
        let l = 1;

        //Setup public parameters
        let pp = PublicParameters::setup(MAX_DEGREE, &mut OsRng).unwrap();

        //Compile circuit
        let mut circuit = SlushieCircuit::<DEPTH>::default();
        let (_dp, dv) = circuit.compile(&pp).unwrap();

        //Calculate commitment
        let commitment = sponge::hash(&[(k as u64).into(), (r as u64).into()]);

        //Calculate opening
        let mut o = [BlsScalar::zero(); DEPTH];
        o[0] = sponge::hash(&[2.into(), 1.into()]);

        o[1] = BlsScalar(bytes_to_u64(hex!(
            "1422626DF22F8FDC85D3F1B54B05DAE703D545326D957C05089191C39D34CB74"
        )));

        //Calculate root
        let n = sponge::hash(&[o[0], commitment]);
        let root = sponge::hash(&[n, o[1]]);

        //Generate proof
        let proof = Proof::from_bytes(
            &prove(
                &pp.to_var_bytes(),
                l,
                u64_to_bytes(root.0),
                [u64_to_bytes(o[0].0), u64_to_bytes(o[1].0)],
                k,
                r,
                PAYOUT,
                RELAYER,
                f,
            )
            .unwrap(),
        );

        // Create public inputs
        let public_inputs: Vec<PublicInputValue> = vec![
            root.into(),
            sponge::hash(&[(k as u64).into()]).into(),
            BlsScalar::from_raw(bytes_to_u64(PAYOUT)).into(),
            BlsScalar::from_raw(bytes_to_u64(RELAYER)).into(),
            BlsScalar::from(f).into(),
        ];

        SlushieCircuit::<DEPTH>::verify(&pp, &dv, &proof.unwrap(), &public_inputs, TRANSCRIPT_INIT)
            .unwrap();
    }

    /// Test for this situation but with wrong index:
    ///         R
    ///        / \
    ///       n   o[1]
    ///      / \
    ///   o[0]  hash(k || r)
    ///    0        1
    #[test]
    #[should_panic = "ProofVerificationError"]
    fn verification_error() {
        //Set depth
        const DEPTH: usize = 2;

        //Set input examples
        const PAYOUT: Pubkey =
            hex!("38c4c4c0f0e9de905b304b60f3ab77b47e2f6b4a388b7859373c6e6a1581708a");
        const RELAYER: Pubkey =
            hex!("92fba99dfb7832c4268e299efb9cd3aaad7153bbee9974729340b528d276936e");
        let k = 3141592653;
        let r = 1;
        let f = 0;
        // Wrong index
        let l = 0;

        //Setup public parameters
        let pp = PublicParameters::setup(MAX_DEGREE, &mut OsRng).unwrap();

        //Compile circuit
        let mut circuit = SlushieCircuit::<DEPTH>::default();
        let (_dp, dv) = circuit.compile(&pp).unwrap();

        //Calculate commitment
        let commitment = sponge::hash(&[(k as u64).into(), (r as u64).into()]);

        //Calculate opening
        let mut o = [BlsScalar::zero(); DEPTH];
        o[0] = sponge::hash(&[2.into(), 1.into()]);

        o[1] = BlsScalar(bytes_to_u64(hex!(
            "1422626DF22F8FDC85D3F1B54B05DAE703D545326D957C05089191C39D34CB74"
        )));

        //Calculate root
        let n = sponge::hash(&[o[0], commitment]);
        let root = sponge::hash(&[n, o[1]]);

        //Generate proof
        let proof = Proof::from_bytes(
            &prove(
                &pp.to_var_bytes(),
                l,
                u64_to_bytes(root.0),
                [u64_to_bytes(o[0].0), u64_to_bytes(o[1].0)],
                k,
                r,
                PAYOUT,
                RELAYER,
                f,
            )
            .unwrap(),
        );

        // Create public inputs
        let public_inputs: Vec<PublicInputValue> = vec![
            root.into(),
            sponge::hash(&[(k as u64).into()]).into(),
            BlsScalar::from_raw(bytes_to_u64(PAYOUT)).into(),
            BlsScalar::from_raw(bytes_to_u64(RELAYER)).into(),
            BlsScalar::from(f).into(),
        ];

        SlushieCircuit::<DEPTH>::verify(&pp, &dv, &proof.unwrap(), &public_inputs, TRANSCRIPT_INIT)
            .expect("ProofVerificationError");
    }

    ///Setup function for every test
    fn setup<const DEPTH: usize>(
        k: u32,
        r: u32,
        l: usize,
    ) -> (
        PublicParameters,
        VerifierData,
        PoseidonHash,
        [PoseidonHash; DEPTH],
    ) {
        //Setup public parameters from file to reduce tests time
        let pp = PublicParameters::from_slice(include_bytes!("../pp-test")).unwrap();

        //Compile circuit
        let mut circuit = SlushieCircuit::<DEPTH>::default();
        let (_dp, dv) = circuit.compile(&pp).unwrap();

        //Calculate commitment
        let commitment = sponge::hash(&[(k as u64).into(), (r as u64).into()]);

        //Calculate opening
        let (R, o) = get_opening(l, commitment);

        // Return public parameters, verifier data, root hash and opening
        (pp, dv, R, o)
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

    /// Merkle tree maximum depth
    use shared::constants::MAX_DEPTH;

    ///Test for checking circuit works with maximum depth(32)
    #[test]
    fn max_depth() {
        // Max depth
        const DEPTH: usize = MAX_DEPTH;

        let k = rand::random::<u32>();
        let r = rand::random::<u32>();
        let l = rand::random::<u32>() as usize;
        let f = rand::random::<u64>();

        let (pp, dv, R, o) = setup::<DEPTH>(k, r, l);

        let proof = Proof::from_bytes(
            &prove(&pp.to_var_bytes(), l, R, o, k, r, PAYOUT, RELAYER, f).unwrap(),
        );

        let public_inputs: Vec<PublicInputValue> = vec![
            BlsScalar(bytes_to_u64(R)).into(),
            sponge::hash(&[(k as u64).into()]).into(),
            BlsScalar::from_raw(bytes_to_u64(PAYOUT)).into(),
            BlsScalar::from_raw(bytes_to_u64(RELAYER)).into(),
            BlsScalar::from(f).into(),
        ];

        SlushieCircuit::<DEPTH>::verify(&pp, &dv, &proof.unwrap(), &public_inputs, TRANSCRIPT_INIT)
            .unwrap();
    }

    ///Test for checking circuit works with random arguments
    #[test]
    fn random_args() {
        const DEPTH: usize = DEFAULT_DEPTH;

        // All arguments are random
        let k = rand::random::<u32>();
        let r = rand::random::<u32>();
        let l = rand::random::<u16>() as usize;
        let f = rand::random::<u64>();

        let (pp, dv, R, o) = setup::<DEPTH>(k, r, l);

        let proof = Proof::from_bytes(
            &prove(&pp.to_var_bytes(), l, R, o, k, r, PAYOUT, RELAYER, f).unwrap(),
        );

        let public_inputs: Vec<PublicInputValue> = vec![
            BlsScalar(bytes_to_u64(R)).into(),
            sponge::hash(&[(k as u64).into()]).into(),
            BlsScalar::from_raw(bytes_to_u64(PAYOUT)).into(),
            BlsScalar::from_raw(bytes_to_u64(RELAYER)).into(),
            BlsScalar::from(f).into(),
        ];

        SlushieCircuit::<DEPTH>::verify(&pp, &dv, &proof.unwrap(), &public_inputs, TRANSCRIPT_INIT)
            .unwrap();
    }

    ///Test for checking circuit works with wrong fee
    #[test]
    #[should_panic = "WrongIndex"]
    fn wrong_index() {
        const DEPTH: usize = 3;
        let k = rand::random::<u32>();
        let r = rand::random::<u32>();
        // Index is more than can be in tree with depth = 3
        let l = 8;
        let f = rand::random::<u64>();

        let (pp, dv, R, o) = setup::<DEPTH>(k, r, l);

        let proof = Proof::from_bytes(
            &prove(&pp.to_var_bytes(), l, R, o, k, r, PAYOUT, RELAYER, f).unwrap(),
        );

        let public_inputs: Vec<PublicInputValue> = vec![
            BlsScalar(bytes_to_u64(R)).into(),
            sponge::hash(&[(k as u64).into()]).into(),
            BlsScalar::from_raw(bytes_to_u64(PAYOUT)).into(),
            BlsScalar::from_raw(bytes_to_u64(RELAYER)).into(),
            BlsScalar::from(f).into(),
        ];

        SlushieCircuit::<DEPTH>::verify(&pp, &dv, &proof.unwrap(), &public_inputs, TRANSCRIPT_INIT)
            .expect("ProofVerificationError");
    }

    ///Test for checking circuit works with wrong fee
    #[test]
    #[should_panic = "ProofVerificationError"]
    fn wrong_fee() {
        const DEPTH: usize = 9;
        let k = rand::random::<u32>();
        let r = rand::random::<u32>();
        let l = rand::random::<u8>() as usize;
        let f = rand::random::<u64>();

        let (pp, dv, R, o) = setup::<DEPTH>(k, r, l);

        let proof = Proof::from_bytes(
            &prove(&pp.to_var_bytes(), l, R, o, k, r, PAYOUT, RELAYER, f).unwrap(),
        );

        let public_inputs: Vec<PublicInputValue> = vec![
            BlsScalar(bytes_to_u64(R)).into(),
            sponge::hash(&[(k as u64).into()]).into(),
            BlsScalar::from_raw(bytes_to_u64(PAYOUT)).into(),
            BlsScalar::from_raw(bytes_to_u64(RELAYER)).into(),
            // Fee in public inputs is incorrect
            BlsScalar::from(f + 1).into(),
        ];

        SlushieCircuit::<DEPTH>::verify(&pp, &dv, &proof.unwrap(), &public_inputs, TRANSCRIPT_INIT)
            .expect("ProofVerificationError");
    }

    ///Test for checking circuit works with wrong nullifier hash
    #[test]
    #[should_panic = "ProofVerificationError"]
    fn wrong_nullifier_hash() {
        const DEPTH: usize = 10;
        let k = rand::random::<u32>();
        let r = rand::random::<u32>();
        let l = rand::random::<u8>() as usize;
        let f = rand::random::<u64>();

        let (pp, dv, R, o) = setup::<DEPTH>(k, r, l);

        let proof = Proof::from_bytes(
            &prove(&pp.to_var_bytes(), l, R, o, k, r, PAYOUT, RELAYER, f).unwrap(),
        );

        let public_inputs: Vec<PublicInputValue> = vec![
            BlsScalar(bytes_to_u64(R)).into(),
            // Nullifier hash in public inputs is incorrect
            BlsScalar::zero().into(),
            BlsScalar::from_raw(bytes_to_u64(PAYOUT)).into(),
            BlsScalar::from_raw(bytes_to_u64(RELAYER)).into(),
            BlsScalar::from(f).into(),
        ];

        SlushieCircuit::<DEPTH>::verify(&pp, &dv, &proof.unwrap(), &public_inputs, TRANSCRIPT_INIT)
            .expect("ProofVerificationError");
    }

    ///Test for checking circuit works with wrong relayer
    #[test]
    #[should_panic = "ProofVerificationError"]
    fn wrong_relayer() {
        const DEPTH: usize = 29;
        let k = rand::random::<u32>();
        let r = rand::random::<u32>();
        let l = rand::random::<u16>() as usize;
        let f = rand::random::<u64>();

        let (pp, dv, R, o) = setup::<DEPTH>(k, r, l);

        let proof = Proof::from_bytes(
            &prove(&pp.to_var_bytes(), l, R, o, k, r, PAYOUT, RELAYER, f).unwrap(),
        );

        let public_inputs: Vec<PublicInputValue> = vec![
            BlsScalar(bytes_to_u64(R)).into(),
            sponge::hash(&[(k as u64).into()]).into(),
            BlsScalar::from_raw(bytes_to_u64(PAYOUT)).into(),
            // Relayer in public inputs is incorrect
            BlsScalar::from_raw(bytes_to_u64(PAYOUT)).into(),
            BlsScalar::from(f).into(),
        ];

        SlushieCircuit::<DEPTH>::verify(&pp, &dv, &proof.unwrap(), &public_inputs, TRANSCRIPT_INIT)
            .expect("ProofVerificationError");
    }

    ///Test for checking circuit works with wrong payout
    #[test]
    #[should_panic = "ProofVerificationError"]
    fn wrong_payout() {
        const DEPTH: usize = 20;
        let k = rand::random::<u32>();
        let r = rand::random::<u32>();
        let l = rand::random::<u16>() as usize;
        let f = rand::random::<u64>();

        let (pp, dv, R, o) = setup::<DEPTH>(k, r, l);

        let proof = Proof::from_bytes(
            &prove(&pp.to_var_bytes(), l, R, o, k, r, PAYOUT, RELAYER, f).unwrap(),
        );

        let public_inputs: Vec<PublicInputValue> = vec![
            BlsScalar(bytes_to_u64(R)).into(),
            sponge::hash(&[(k as u64).into()]).into(),
            // Payout in public inputs is incorrect
            BlsScalar::from_raw(bytes_to_u64(RELAYER)).into(),
            BlsScalar::from_raw(bytes_to_u64(RELAYER)).into(),
            BlsScalar::from(f).into(),
        ];

        SlushieCircuit::<DEPTH>::verify(&pp, &dv, &proof.unwrap(), &public_inputs, TRANSCRIPT_INIT)
            .expect("ProofVerificationError");
    }

    ///Test for checking circuit works with wrong root
    #[test]
    #[should_panic = "ProofVerificationError"]
    fn wrong_root() {
        const DEPTH: usize = 10;
        let k = rand::random::<u32>();
        let r = rand::random::<u32>();
        let l = rand::random::<u8>() as usize;
        let f = rand::random::<u64>();

        let (pp, dv, R, o) = setup::<DEPTH>(k, r, l);

        let proof = Proof::from_bytes(
            &prove(&pp.to_var_bytes(), l, R, o, k, r, PAYOUT, RELAYER, f).unwrap(),
        );

        let public_inputs: Vec<PublicInputValue> = vec![
            // Root in public inputs is incorrect
            BlsScalar::one().into(),
            sponge::hash(&[(k as u64).into()]).into(),
            BlsScalar::from_raw(bytes_to_u64(RELAYER)).into(),
            BlsScalar::from_raw(bytes_to_u64(PAYOUT)).into(),
            BlsScalar::from(f).into(),
        ];

        SlushieCircuit::<DEPTH>::verify(&pp, &dv, &proof.unwrap(), &public_inputs, TRANSCRIPT_INIT)
            .expect("ProofVerificationError");
    }

    ///Test for checking circuit works with wrong opening
    #[test]
    #[should_panic = "ProofVerificationError"]
    fn wrong_opening_with_small_depth() {
        const DEPTH: usize = 4;
        let k = rand::random::<u32>();
        let r = rand::random::<u32>();
        let l = 10usize;
        let f = rand::random::<u64>();

        let (pp, dv, R, mut o) = setup::<DEPTH>(k, r, l);

        // Opening is incorrect
        o[1] = [0; 32];

        let proof = Proof::from_bytes(
            &prove(&pp.to_var_bytes(), l, R, o, k, r, PAYOUT, RELAYER, f).unwrap(),
        );

        let public_inputs: Vec<PublicInputValue> = vec![
            BlsScalar(bytes_to_u64(R)).into(),
            sponge::hash(&[(k as u64).into()]).into(),
            BlsScalar::from_raw(bytes_to_u64(RELAYER)).into(),
            BlsScalar::from_raw(bytes_to_u64(PAYOUT)).into(),
            BlsScalar::from(f).into(),
        ];

        SlushieCircuit::<DEPTH>::verify(&pp, &dv, &proof.unwrap(), &public_inputs, TRANSCRIPT_INIT)
            .expect("ProofVerificationError");
    }

    ///Test for checking circuit works with wrong opening
    #[test]
    #[should_panic = "PolynomialDegreeTooLarge"]
    fn wrong_opening_with_big_depth() {
        const DEPTH: usize = MAX_DEPTH;
        let k = rand::random::<u32>();
        let r = rand::random::<u32>();
        let l = rand::random::<u32>() as usize;
        let f = rand::random::<u64>();

        let (pp, dv, R, mut o) = setup::<DEPTH>(k, r, l);

        // Opening is incorrect
        o[10] = [0; 32];

        // For generating proof with incorrect opening, circuit needs more degree and time,
        // but ProofVerificationError will be in result. To decrease tests time we use
        // PolynomialDegreeTooLarge error during generating proof with incorrect data
        let proof = Proof::from_bytes(
            &prove(&pp.to_var_bytes(), l, R, o, k, r, PAYOUT, RELAYER, f)
                .expect("PolynomialDegreeTooLarge"),
        );

        let public_inputs: Vec<PublicInputValue> = vec![
            BlsScalar(bytes_to_u64(R)).into(),
            sponge::hash(&[(k as u64).into()]).into(),
            BlsScalar::from_raw(bytes_to_u64(RELAYER)).into(),
            BlsScalar::from_raw(bytes_to_u64(PAYOUT)).into(),
            BlsScalar::from(f).into(),
        ];

        SlushieCircuit::<DEPTH>::verify(&pp, &dv, &proof.unwrap(), &public_inputs, TRANSCRIPT_INIT)
            .unwrap();
    }

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    ///Test for checking circuit works in wasm
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test]
    fn generate_proof_test() {
        const DEPTH: usize = DEFAULT_DEPTH;
        let k = rand::random::<u32>();
        let r = rand::random::<u32>();
        let l = rand::random::<u16>() as usize;
        let f = rand::random::<u64>();

        let (pp, dv, R, o) = setup::<DEPTH>(k, r, l);

        let proof = Proof::from_bytes(
            &generate_proof(
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
            .unwrap(),
        );

        let public_inputs: Vec<PublicInputValue> = vec![
            BlsScalar(bytes_to_u64(R)).into(),
            sponge::hash(&[(k as u64).into()]).into(),
            BlsScalar::from_raw(bytes_to_u64(PAYOUT)).into(),
            BlsScalar::from_raw(bytes_to_u64(RELAYER)).into(),
            BlsScalar::from(f).into(),
        ];

        SlushieCircuit::<DEPTH>::verify(&pp, &dv, &proof.unwrap(), &public_inputs, TRANSCRIPT_INIT)
            .unwrap();
    }
}
