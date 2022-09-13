use alloc::vec::Vec;

use dusk_bls12_381::BlsScalar;
use dusk_bytes::Serializable;
use dusk_plonk::prelude::*;
use shared::functions::bytes_to_u64;

use crate::circuit::*;
use shared::public_types::*;

///Verification serialized proof in cases when public parameters is available
///Depth can be custom
#[allow(non_snake_case)]
pub fn verify<const DEPTH: usize>(
    //Public parameters
    pp: &[u8],
    //Nullifier hash
    h: PoseidonHash,
    //Root
    R: PoseidonHash,
    //Recipient address
    A: Pubkey,
    //Relayer address
    t: Pubkey,
    //Fee
    f: u64,
    //Proof
    proof: &SerializedProof,
) -> Result<(), Error> {
    //Read public parameters
    let pp = PublicParameters::from_slice(pp)?;

    //Compile circuit
    let mut circuit = SlushieCircuit::<DEPTH>::default();
    let (_pk, vd) = circuit.compile(&pp)?;

    // Proof deserialization
    let proof = Proof::from_bytes(proof)?;

    // Create public inputs
    let public_inputs: Vec<PublicInputValue> = vec![
        BlsScalar(bytes_to_u64(R)).into(),
        BlsScalar(bytes_to_u64(h)).into(),
        BlsScalar::from_raw(bytes_to_u64(A)).into(),
        BlsScalar::from_raw(bytes_to_u64(t)).into(),
        BlsScalar::from(f).into(),
    ];

    // Verify proof using public inputs
    SlushieCircuit::<DEPTH>::verify(&pp, &vd, &proof, &public_inputs, TRANSCRIPT_INIT)
}

///Verification serialized proof in cases when public parameters is too large
///Only default tree depth
#[allow(clippy::too_many_arguments)]
#[allow(non_snake_case)]
pub fn verify_with_vd(
    //Verifier data
    vd: &[u8],
    //Opening key
    opening_key: &[u8; OpeningKey::SIZE],
    //Nullifier hash
    h: PoseidonHash,
    //Root
    R: PoseidonHash,
    //Recipient address
    A: Pubkey,
    //Relayer address
    t: Pubkey,
    //Fee
    f: u64,
    //Proof
    proof: &SerializedProof,
) -> Result<(), Error> {
    // Verifier data deserialization
    let vd = VerifierData::from_slice(vd)?;

    //Opening key deserialization
    let opening_key = OpeningKey::from_bytes(opening_key)?;

    // Proof deserialization
    let proof = Proof::from_bytes(proof)?;

    // Setup for verifier
    let mut verifier = Verifier::new(TRANSCRIPT_INIT);
    verifier.verifier_key.replace(*vd.key());

    let pi_indexes = vd.public_inputs_indexes();
    let public_inputs = [
        BlsScalar::zero(),
        BlsScalar::zero(),
        BlsScalar::zero(),
        -BlsScalar(bytes_to_u64(R)),
        -BlsScalar(bytes_to_u64(h)),
        -BlsScalar::from_raw(bytes_to_u64(A)),
        -BlsScalar::from_raw(bytes_to_u64(t)),
        -BlsScalar::from(f),
    ];

    verifier.verify(&proof, &opening_key, &public_inputs, pi_indexes)
}
