#![cfg_attr(not(feature = "std"), no_std)]

use dusk_bytes::Serializable;
use dusk_plonk::prelude::*;
use dusk_poseidon::sponge;
use rand_core::OsRng;

type PoseidonHash = [u8; 32];
type Pubkey = [u8; 32];

const CIRCUIT_SIZE: usize = 1 << 10;
const TRANSCRIPT_INIT: &[u8; 7] = b"slushie";

///Circuit that checks:
/// 1) poseidonHash(k) = h where h is a Public Input
#[derive(Debug, Default)]
pub struct SlushieCircuit {
    ///Private
    ///Nullifier
    k: BlsScalar,

    ///Public
    ///Nullifier hash
    h: BlsScalar,
    ///Recipient address
    a: BlsScalar,
    ///Relayer address
    t: BlsScalar,
    ///Fee
    f: BlsScalar,
}

impl Circuit for SlushieCircuit {
    const CIRCUIT_ID: [u8; 32] = [0xff; 32];
    fn gadget(&mut self, composer: &mut TurboComposer) -> core::result::Result<(), Error> {
        //Add secret elements to composer
        let k = composer.append_witness(self.k);

        //Add public elements to composer
        let hash = composer.append_public_witness(self.h);
        composer.append_public_witness(self.a);
        composer.append_public_witness(self.t);
        composer.append_public_witness(self.f);

        //Compute poseidon hash of nullifier
        let computed_hash = sponge::gadget(composer, &[k]);

        //Add equal gate
        composer.assert_equal(hash, computed_hash);

        Ok(())
    }

    fn public_inputs(&self) -> Vec<PublicInputValue> {
        vec![self.h.into(), self.a.into(), self.t.into(), self.f.into()]
    }

    fn padded_gates(&self) -> usize {
        CIRCUIT_SIZE
    }
}

///Generation serialized proof
#[allow(clippy::too_many_arguments)]
pub fn prove(
    //Public parameters
    pp: Vec<u8>,
    //Leaf index
    _l: usize,
    //Tree opening
    _o: Vec<PoseidonHash>,
    //Nullifier
    k: u32,
    //Randomness
    _r: u32,
    //Recipient address
    a: Pubkey,
    //Relayer address
    t: Pubkey,
    //Fee
    f: u64,
) -> Result<[u8; 1040], Error> {
    //Read public parameters
    let pp = PublicParameters::from_slice(&pp[..])?;

    //Compile circuit
    let mut circuit = SlushieCircuit::default();
    let (pk, _vd) = circuit.compile(&pp)?;

    //Create circuit
    let mut circuit = SlushieCircuit {
        k: (k as u64).into(),
        h: sponge::hash(&[(k as u64).into()]),
        a: BlsScalar::from_raw(bytes_to_u64(a)),
        t: BlsScalar::from_raw(bytes_to_u64(t)),
        f: f.into(),
    };

    //Generate proof
    circuit
        .prove(&pp, &pk, TRANSCRIPT_INIT, &mut OsRng)
        .map(|proof| proof.to_bytes())
}

pub fn bytes_to_u64(bytes: [u8; 32]) -> [u64; 4] {
    let mut result = [0; 4];

    for i in 0..result.len() {
        let bytes_8 = bytes.split_at(i * 8).1.split_at(8).0;
        let bytes_array = <&[u8; 8]>::try_from(bytes_8).unwrap();
        result[i] = u64::from_be_bytes(*bytes_array);
    }

    result
}

pub fn u64_to_bytes(array: [u64; 4]) -> [u8; 32] {
    let mut result = [0; 32];

    for i in 0..array.len() {
        let bytes_array = array[i].to_be_bytes();
        for j in 0..bytes_array.len() {
            result[i * 8 + j] = bytes_array[j];
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use dusk_plonk::prelude::*;
    use dusk_poseidon::sponge;
    use hex_literal::hex;

    use super::*;

    const MAX_DEGREE: usize = CIRCUIT_SIZE << 1;

    #[test]
    fn confirm_proof() {
        const PAYOUT: Pubkey =
            hex!("38c4c4c0f0e9de905b304b60f3ab77b47e2f6b4a388b7859373c6e6a1581708a");
        const RELAYER: Pubkey =
            hex!("92fba99dfb7832c4268e299efb9cd3aaad7153bbee9974729340b528d276936e");

        let pp = PublicParameters::setup(MAX_DEGREE, &mut OsRng).unwrap();

        let mut circuit = SlushieCircuit::default();
        let (_dp, dv) = circuit.compile(&pp).unwrap();

        let proof = Proof::from_bytes(
            &prove(
                pp.to_var_bytes(),
                0,
                vec![],
                3141592653,
                0,
                PAYOUT,
                RELAYER,
                0,
            )
            .unwrap(),
        );

        let public_inputs: Vec<PublicInputValue> = vec![
            sponge::hash(&[3141592653.into()]).into(),
            BlsScalar::from_raw(bytes_to_u64(PAYOUT)).into(),
            BlsScalar::from_raw(bytes_to_u64(RELAYER)).into(),
            BlsScalar::from(0).into(),
        ];

        SlushieCircuit::verify(&pp, &dv, &proof.unwrap(), &public_inputs, TRANSCRIPT_INIT).unwrap();
    }
}
