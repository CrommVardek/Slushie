#![cfg(feature = "proof_generator")]
use crate::circuit::*;
use crate::utils::*;
use dusk_bytes::Serializable;
use shared::constants::DEFAULT_DEPTH;
use shared::functions::bytes_to_u64;
use shared::public_types::*;

use dusk_plonk::prelude::*;
use dusk_poseidon::sponge;

use rand_core::OsRng;

///Generate serialized proof in cases when public parameters is available
///Depth can be custom
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
) -> Result<SerializedProof, Error> {
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

///Generate serialized proof in cases when compilation with public parameters is too long
///Only default tree depth
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
#[allow(non_snake_case)]
pub fn prove_with_vd(
    //Prover data
    pd: &[u8],
    //Commitment key
    ck: &[u8],
    //Leaf index
    l: usize,
    //Root
    R: PoseidonHash,
    //Tree opening
    o: [PoseidonHash; DEFAULT_DEPTH],
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
) -> Result<SerializedProof, Error> {
    //Read prover data and commit key
    let pd = ProverKey::from_slice(pd)?;
    let ck = CommitKey::from_slice(ck)?;

    // New Prover instance
    let mut prover = Prover::new(TRANSCRIPT_INIT);

    //Create circuit
    let mut circuit = SlushieCircuit::<DEFAULT_DEPTH> {
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

    // Fill witnesses for Prover
    circuit.gadget(prover.composer_mut())?;

    // Add prover data to Prover
    prover.prover_key = Some(pd);
    prover.prove(&ck, &mut OsRng).map(|proof| proof.to_bytes())
}
