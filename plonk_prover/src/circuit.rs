use crate::utils::Array;

use alloc::vec::Vec;
use dusk_bytes::Serializable;
use dusk_plonk::prelude::*;
use dusk_poseidon::sponge;
use shared::functions::bytes_to_u64;

pub type PoseidonHash = [u8; 32];
pub type Pubkey = [u8; 32];
pub type SerializedProof = [u8; Proof::SIZE];

pub(crate) const CIRCUIT_SIZE: usize = 1 << 16;

///Constant which should be equal during generating proof and verifying its
pub(crate) const TRANSCRIPT_INIT: &[u8; 7] = b"slushie";

/// Circuit that checks:
/// 1) poseidonHash(k) = h where h is a Public Input
/// 2) root of tree opening and commitment = R where R is a Public Input
#[allow(non_snake_case)]
#[derive(Debug, Default)]
pub(crate) struct SlushieCircuit<const DEPTH: usize> {
    ///Private
    ///Nullifier
    pub k: BlsScalar,
    ///Randomness
    pub r: BlsScalar,
    ///Tree opening
    pub o: Array<PoseidonHash, DEPTH>,
    ///Tree path
    pub p: Array<u8, DEPTH>,

    ///Public
    ///Root
    pub R: BlsScalar,
    ///Nullifier hash
    pub h: BlsScalar,
    ///Recipient address
    pub A: BlsScalar,
    ///Relayer address
    pub t: BlsScalar,
    ///Fee
    pub f: BlsScalar,
}

impl<const DEPTH: usize> Circuit for SlushieCircuit<DEPTH> {
    const CIRCUIT_ID: [u8; 32] = [0xff; 32];
    fn gadget(&mut self, composer: &mut TurboComposer) -> core::result::Result<(), Error> {
        //Add secret elements to composer
        let k = composer.append_witness(self.k);
        let r = composer.append_witness(self.r);
        let mut path = [composer.append_witness(BlsScalar::zero()); DEPTH];
        for (i, path) in path.iter_mut().enumerate().take(DEPTH) {
            *path = composer.append_witness(self.p.0[i] as u64);
        }

        //Add public elements to composer
        let root = composer.append_public_witness(self.R);
        let nullifier_hash = composer.append_public_witness(self.h);
        composer.append_public_witness(self.A);
        composer.append_public_witness(self.t);
        composer.append_public_witness(self.f);

        //Compute poseidon hash of nullifier
        let computed_nullifier_hash = sponge::gadget(composer, &[k]);

        //Compute poseidon hash of (nullifier || randomness)
        let computed_commitment = sponge::gadget(composer, &[k, r]);

        //Compute all hashes on the path
        let mut hashes = [composer.append_witness(BlsScalar::zero()); DEPTH];
        for i in 0..DEPTH {
            let current_hash = if i == 0 {
                computed_commitment
            } else {
                hashes[i - 1]
            };
            let sister_hash = composer.append_witness(BlsScalar(bytes_to_u64(self.o.0[i])));

            let left = get_left_right(composer, sister_hash, current_hash, path[i]);
            let right = get_left_right(composer, current_hash, sister_hash, path[i]);

            hashes[i] = sponge::gadget(composer, &[left, right]);
        }

        //Add equal gates
        composer.assert_equal(root, hashes[DEPTH - 1]);
        composer.assert_equal(nullifier_hash, computed_nullifier_hash);

        Ok(())
    }

    fn public_inputs(&self) -> Vec<PublicInputValue> {
        vec![
            self.R.into(),
            self.h.into(),
            self.A.into(),
            self.t.into(),
            self.f.into(),
        ]
    }

    fn padded_gates(&self) -> usize {
        CIRCUIT_SIZE
    }
}

/// Function to get left and right hashes using path
/// z = (x - y) * p + y
fn get_left_right(composer: &mut TurboComposer, x: Witness, y: Witness, p: Witness) -> Witness {
    //(x - y)
    let sum_constraint = Constraint::new().left(1).a(x).right(-BlsScalar::one()).b(y);
    let sum = composer.gate_add(sum_constraint);

    //(x - y) * p
    let mul_constraint = Constraint::new().mult(1).a(p).b(sum);
    let mul = composer.gate_add(mul_constraint);

    //(x - y) * p + y
    let sum_constraint = Constraint::new().left(1).a(mul).right(1).b(y);
    composer.gate_add(sum_constraint)
}
