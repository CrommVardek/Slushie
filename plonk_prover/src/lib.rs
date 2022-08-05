#![cfg_attr(not(feature = "std"), no_std)]

mod circuit;
mod utils;

use circuit::*;
use utils::*;

use dusk_bytes::Serializable;
use dusk_plonk::prelude::*;
use dusk_poseidon::sponge;
use rand_core::OsRng;

const TRANSCRIPT_INIT: &[u8; 7] = b"slushie";

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(target_arch = "wasm32")]
use core::array::TryFromSliceError;

#[cfg(target_arch = "wasm32")]
const DEFAULT_DEPTH: usize = 32;

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
        .map_err(|err| js_sys::Error::new(&err.to_string()))
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
        p: Array(index_to_path(l)),
    };

    //Generate proof
    circuit
        .prove(&pp, &pk, TRANSCRIPT_INIT, &mut OsRng)
        .map(|proof| proof.to_bytes())
}

/// Tests take some time due to proof generating. Recommend running them in release mode
#[cfg(test)]
mod tests {
    use dusk_plonk::prelude::*;
    use dusk_poseidon::sponge;
    use hex_literal::hex;

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
    #[should_panic]
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
            .unwrap();
    }
}
