use hex::ToHex;
use plonk_prover::public_parameters_generation::*;
use plonk_prover::{prove, GeneratedCommitment};
use sp_core::crypto::{AccountId32, Ss58Codec};

use crate::utils::{read_pp, write_to_file};
use crate::{commands::Commands, utils::parse_tree_openings};

/// Generate proof and write it to file
pub(crate) fn generate_proof(args: &Commands) {
    // Get arguments from command
    let (pp, l, root, o, k, r, a, t, f, output_file) = if let Commands::GenerateProof {
        pp,
        l,
        root,
        o,
        k,
        r,
        a,
        t,
        f,
        output_file,
    } = args
    {
        (pp, l, root, o, k, r, a, t, f, output_file)
    } else {
        panic!("Wrong Command!")
    };

    // Read serialized pp
    let pp_bytes = read_pp(pp);

    // Read and convert recipient address
    let a = AccountId32::from_ss58check(a)
        .expect("Could not convert input to AccountId32")
        .into();

    // Read and convert relayer address
    let t = AccountId32::from_ss58check(t)
        .expect("Could not convert input to AccountId32")
        .into();

    // Read and parse tree opening
    let o = parse_tree_openings(o);

    // Read and parse root
    let root: [u8; 32] = hex::decode(root).unwrap().try_into().unwrap();

    // Generate proof
    let proof = prove(&pp_bytes, *l, root, o, *k, *r, a, t, *f).expect("Error generating proof");

    // Write serialized proof to file
    write_to_file(output_file, &proof);

    println!("Success! Your proof generated in {}!", output_file);
    println!("You can use Proof to call withdraw contract method");
}

pub fn generate_commitment() {
    // Generate randomness, nullifier, commitment and nullifier hash
    let GeneratedCommitment {
        nullifier,
        randomness,
        commitment_bytes,
        nullifier_hash_bytes,
    } = plonk_prover::generate_commitment();

    // Convert commitment bytes to hex
    let hex_commitment = commitment_bytes.encode_hex_upper::<String>();

    // Convert nullifier hash bytes to hex
    let hex_nullifier_hash = nullifier_hash_bytes.encode_hex_upper::<String>();

    println!("Successfully generated! Please save this values:");
    println!("Nullifier: {}", nullifier);
    println!("Randomness: {}", randomness);
    println!("Commitment: {}", hex_commitment);
    println!("Nullifier Hash: {}", hex_nullifier_hash);
    println!("You can use:");
    println!(" • commitment to call deposit contract method");
    println!(" • randomness, nullifier and nullifier hash to generate your Proof");
    println!(" • nullifier hash to call withdraw contract method")
}

/// Generate public parameters
pub fn generate_pp(output_file: &str) {
    let pp_bytes = generate_test_public_parameters().expect("Could not generate public parameters");

    write_to_file(output_file, &pp_bytes);

    println!("Successfully generated!");
}

/// Generate prover data for provided public parameters
pub fn generate_pd(args: &Commands) {
    // Get arguments from command
    let (pp, output_pd, output_ck) = if let Commands::GenerateProverData {
        pp,
        output_pd,
        output_ck,
    } = args
    {
        (pp, output_pd, output_ck)
    } else {
        panic!("Wrong Command!")
    };

    let pp_bytes = read_pp(pp);

    let (pd, ck) = generate_prover_data(&pp_bytes).expect("Could not generate prover data");

    write_to_file(output_pd, &pd);
    write_to_file(output_ck, &ck);

    println!("Successfully generated!");
}

/// Generate verifier data for provided public parameters
pub fn generate_vd(args: &Commands) {
    // Get arguments from command
    let (pp, output_vd, output_ok) = if let Commands::GenerateVerifierData {
        pp,
        output_vd,
        output_ok,
    } = args
    {
        (pp, output_vd, output_ok)
    } else {
        panic!("Wrong Command!")
    };

    let pp_bytes = read_pp(pp);

    let (vd, ok) = generate_verifier_data(&pp_bytes).expect("Could not generate verifier data");

    write_to_file(output_vd, &vd);
    write_to_file(output_ok, &ok);

    println!("Successfully generated!");
}
