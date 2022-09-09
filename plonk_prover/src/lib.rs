#![cfg_attr(not(feature = "std"), no_std)]

mod circuit;
mod commitment_generation;
pub mod hasher;
mod proof_generation;
mod proof_verification;
mod utils;

#[cfg(feature = "proof_generator")]
pub mod merkle_tree;

#[macro_use]
extern crate alloc;

#[cfg(feature = "proof_generator")]
pub use proof_generation::prove;

#[cfg(feature = "proof_generator")]
pub use utils::index_to_path;

#[cfg(feature = "proof_generator")]
pub use commitment_generation::{generate_commitment, GeneratedCommitment};

pub use proof_verification::*;

pub use circuit::{PoseidonHash, Pubkey};

/// Tests take some time due to proof generating. Recommend running them in release mode with parallel feature
/// cargo test -r --features parallel  
#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use dusk_bytes::Serializable;
    use dusk_plonk::prelude::*;
    use dusk_poseidon::sponge;
    use hex_literal::hex;
    use rand_core::OsRng;
    use shared::functions::bytes_to_u64;
    use shared::functions::scalar_to_bytes;
    use shared::functions::u64_to_bytes;

    use crate::circuit::*;
    use crate::proof_generation::prove;
    use crate::utils::index_to_path;

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

        //Calculate nullifier hash
        let h = sponge::hash(&[(k as u64).into()]);

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
        let proof = &prove(
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
        .unwrap();

        // Verify proof
        verify::<DEPTH>(
            &pp.to_var_bytes(),
            scalar_to_bytes(h),
            scalar_to_bytes(root),
            PAYOUT,
            RELAYER,
            f,
            proof,
        )
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

        //Calculate nullifier hash
        let h = sponge::hash(&[(k as u64).into()]);

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
        let proof = &prove(
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
        .unwrap();

        // Verify proof
        verify::<DEPTH>(
            &pp.to_var_bytes(),
            scalar_to_bytes(h),
            scalar_to_bytes(root),
            PAYOUT,
            RELAYER,
            f,
            proof,
        )
        .unwrap();
    }

    ///Setup function for every test
    fn setup<'a, const DEPTH: usize>(
        k: u32,
        r: u32,
        l: usize,
    ) -> (
        &'a [u8],
        &'a [u8],
        &'a [u8; OpeningKey::SIZE],
        PoseidonHash,
        [PoseidonHash; DEPTH],
    ) {
        //Setup public parameters from file to reduce tests time
        let pp = include_bytes!("../pp-test");

        // Setup verifier data and opening key from file
        let vd = include_bytes!("../vd-test");
        let opening_key = include_bytes!("../op-key-test");

        //Calculate commitment
        let commitment = sponge::hash(&[(k as u64).into(), (r as u64).into()]);

        //Calculate opening
        let (R, o) = get_opening(l, commitment);

        // Return public parameters, verifier data, root hash and opening
        (pp, vd, opening_key, R, o)
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

        let (pp, _vd, _opening_key, R, o) = setup::<DEPTH>(k, r, l);

        let proof = &prove(pp, l, R, o, k, r, PAYOUT, RELAYER, f).unwrap();

        let h = sponge::hash(&[(k as u64).into()]);

        verify::<DEPTH>(pp, scalar_to_bytes(h), R, PAYOUT, RELAYER, f, proof).unwrap();
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

        let (pp, vd, opening_key, R, o) = setup::<DEPTH>(k, r, l);

        let proof = &prove(pp, l, R, o, k, r, PAYOUT, RELAYER, f).unwrap();

        let h = sponge::hash(&[(k as u64).into()]);

        verify::<DEPTH>(pp, scalar_to_bytes(h), R, PAYOUT, RELAYER, f, proof).unwrap();
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

        let (pp, vd, opening_key, R, o) = setup::<DEPTH>(k, r, l);

        let proof = &prove(pp, l, R, o, k, r, PAYOUT, RELAYER, f).unwrap();

        let h = sponge::hash(&[(k as u64).into()]);

        verify::<DEPTH>(pp, scalar_to_bytes(h), R, PAYOUT, RELAYER, f, proof)
            .or_else(|_| {
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
            })
            .expect("ProofVerificationError");
    }

    ///Test for checking circuit works with wrong fee
    #[test]
    #[should_panic = "ProofVerificationError"]
    fn wrong_fee() {
        const DEPTH: usize = DEFAULT_DEPTH;
        let k = rand::random::<u32>();
        let r = rand::random::<u32>();
        let l = rand::random::<u8>() as usize;
        let f = rand::random::<u64>();

        let (pp, vd, opening_key, R, o) = setup::<DEPTH>(k, r, l);

        let proof = &prove(pp, l, R, o, k, r, PAYOUT, RELAYER, f).unwrap();

        let h = sponge::hash(&[(k as u64).into()]);

        verify::<DEPTH>(
            pp,
            scalar_to_bytes(h),
            R,
            PAYOUT,
            RELAYER,
            // Fee is incorrect
            f + 1,
            proof,
        )
        .or_else(|_| {
            verify_with_vd(
                vd,
                opening_key,
                scalar_to_bytes(h),
                R,
                PAYOUT,
                RELAYER,
                // Fee is incorrect
                f + 1,
                proof,
            )
        })
        .expect("ProofVerificationError");
    }

    ///Test for checking circuit works with wrong nullifier hash
    #[test]
    #[should_panic = "ProofVerificationError"]
    fn wrong_nullifier_hash() {
        const DEPTH: usize = DEFAULT_DEPTH;
        let k = rand::random::<u32>();
        let r = rand::random::<u32>();
        let l = rand::random::<u8>() as usize;
        let f = rand::random::<u64>();

        let (pp, vd, opening_key, R, o) = setup::<DEPTH>(k, r, l);

        let proof = &prove(pp, l, R, o, k, r, PAYOUT, RELAYER, f).unwrap();

        verify::<DEPTH>(
            pp,
            // Nullifier hash in public inputs is incorrect
            scalar_to_bytes(BlsScalar::zero()),
            R,
            PAYOUT,
            RELAYER,
            f,
            proof,
        )
        .or_else(|_| {
            verify_with_vd(
                vd,
                opening_key,
                // Nullifier hash in public inputs is incorrect
                scalar_to_bytes(BlsScalar::zero()),
                R,
                PAYOUT,
                RELAYER,
                f,
                proof,
            )
        })
        .expect("ProofVerificationError");
    }

    ///Test for checking circuit works with wrong relayer
    #[test]
    #[should_panic = "ProofVerificationError"]
    fn wrong_relayer() {
        const DEPTH: usize = DEFAULT_DEPTH;
        let k = rand::random::<u32>();
        let r = rand::random::<u32>();
        let l = rand::random::<u16>() as usize;
        let f = rand::random::<u64>();

        let (pp, vd, opening_key, R, o) = setup::<DEPTH>(k, r, l);

        let proof = &prove(pp, l, R, o, k, r, PAYOUT, RELAYER, f).unwrap();

        let h = sponge::hash(&[(k as u64).into()]);

        verify::<DEPTH>(
            pp,
            scalar_to_bytes(h),
            R,
            PAYOUT,
            // Relayer in public inputs is incorrect
            PAYOUT,
            f,
            proof,
        )
        .or_else(|_| {
            verify_with_vd(
                vd,
                opening_key,
                scalar_to_bytes(h),
                R,
                PAYOUT,
                // Relayer in public inputs is incorrect
                PAYOUT,
                f,
                proof,
            )
        })
        .expect("ProofVerificationError");
    }

    ///Test for checking circuit works with wrong payout
    #[test]
    #[should_panic = "ProofVerificationError"]
    fn wrong_payout() {
        const DEPTH: usize = DEFAULT_DEPTH;
        let k = rand::random::<u32>();
        let r = rand::random::<u32>();
        let l = rand::random::<u16>() as usize;
        let f = rand::random::<u64>();

        let (pp, vd, opening_key, R, o) = setup::<DEPTH>(k, r, l);

        let proof = &prove(pp, l, R, o, k, r, PAYOUT, RELAYER, f).unwrap();

        let h = sponge::hash(&[(k as u64).into()]);

        verify::<DEPTH>(
            pp,
            scalar_to_bytes(h),
            R,
            // Payout in public inputs is incorrect
            RELAYER,
            RELAYER,
            f,
            proof,
        )
        .or_else(|_| {
            verify_with_vd(
                vd,
                opening_key,
                scalar_to_bytes(h),
                R,
                // Payout in public inputs is incorrect
                RELAYER,
                RELAYER,
                f,
                proof,
            )
        })
        .expect("ProofVerificationError");
    }

    ///Test for checking circuit works with wrong root
    #[test]
    #[should_panic = "ProofVerificationError"]
    fn wrong_root() {
        const DEPTH: usize = DEFAULT_DEPTH;
        let k = rand::random::<u32>();
        let r = rand::random::<u32>();
        let l = rand::random::<u8>() as usize;
        let f = rand::random::<u64>();

        let (pp, vd, opening_key, R, o) = setup::<DEPTH>(k, r, l);

        let proof = &prove(pp, l, R, o, k, r, PAYOUT, RELAYER, f).unwrap();

        let h = sponge::hash(&[(k as u64).into()]);

        verify::<DEPTH>(
            pp,
            scalar_to_bytes(h),
            // Root in public inputs is incorrect
            [0; 32],
            PAYOUT,
            RELAYER,
            f,
            proof,
        )
        .or_else(|_| {
            verify_with_vd(
                vd,
                opening_key,
                scalar_to_bytes(h),
                // Root in public inputs is incorrect
                [0; 32],
                PAYOUT,
                RELAYER,
                f,
                proof,
            )
        })
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

        let (pp, vd, opening_key, R, mut o) = setup::<DEPTH>(k, r, l);

        // Opening is incorrect
        o[1] = [0; 32];

        let proof = &prove(pp, l, R, o, k, r, PAYOUT, RELAYER, f).unwrap();

        let h = sponge::hash(&[(k as u64).into()]);

        verify::<DEPTH>(pp, scalar_to_bytes(h), R, PAYOUT, RELAYER, f, proof)
            .expect("ProofVerificationError");
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

        let (pp, _vd, _opening_key, R, mut o) = setup::<DEPTH>(k, r, l);

        // Opening is incorrect
        o[10] = [0; 32];

        // For generating proof with incorrect opening, circuit needs more degree and time,
        // but ProofVerificationError will be in result. To decrease tests time we use
        // PolynomialDegreeTooLarge error during generating proof with incorrect data
        let proof =
            &prove(pp, l, R, o, k, r, PAYOUT, RELAYER, f).expect("PolynomialDegreeTooLarge");

        let h = sponge::hash(&[(k as u64).into()]);

        verify::<DEPTH>(pp, scalar_to_bytes(h), R, PAYOUT, RELAYER, f, proof).unwrap();
    }
}
