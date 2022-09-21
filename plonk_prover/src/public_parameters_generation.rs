#![cfg(feature = "proof_generator")]

use crate::circuit::{SlushieCircuit, CIRCUIT_SIZE};
use alloc::vec::Vec;
use dusk_bytes::Serializable;
use dusk_plonk::prelude::*;
use rand_core::OsRng;
use shared::constants::DEFAULT_DEPTH;

pub fn generate_test_public_parameters() -> Result<Vec<u8>, Error> {
    PublicParameters::setup(CIRCUIT_SIZE, &mut OsRng).map(|pp| pp.to_var_bytes())
}

pub fn generate_verifier_data(pp: &[u8]) -> Result<(Vec<u8>, [u8; OpeningKey::SIZE]), Error> {
    let pp = PublicParameters::from_slice(pp)?;

    let mut circuit = SlushieCircuit::<DEFAULT_DEPTH>::default();

    let (_, vd) = circuit.compile(&pp)?;

    Ok((vd.to_var_bytes(), pp.opening_key().to_bytes()))
}

pub fn generate_prover_data(pp: &[u8]) -> Result<(Vec<u8>, Vec<u8>), Error> {
    let pp = PublicParameters::from_slice(pp)?;

    let mut circuit = SlushieCircuit::<DEFAULT_DEPTH>::default();

    let (pd, _) = circuit.compile(&pp)?;

    Ok((pd.to_var_bytes(), pp.commit_key().to_var_bytes()))
}
