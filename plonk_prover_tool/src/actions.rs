use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use hex::ToHex;
use plonk_prover::{prove, GeneratedCommitment};
use sp_core::crypto::{AccountId32, Ss58Codec};

use crate::{commands::Commands, utils::parse_tree_openings};

/// generate proof and write it to file
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

    // Read path to pp
    let path = Path::new(&pp);

    // Read serialized pp
    let mut pp_bytes = Vec::new();
    File::open(path)
        .unwrap()
        .read_to_end(&mut pp_bytes)
        .expect("Unable to read Public Parameters from file");

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
    let output_file_path = output_file;
    let mut output_file = File::create(output_file_path).expect("Unable to create file");
    output_file
        .write_all(&proof)
        .expect("Unable to write proof to file");

    println!("Success! Your proof generated in {}!", output_file_path);
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
