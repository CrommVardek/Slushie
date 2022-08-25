use clap::Parser;
use clap::Subcommand;

use crate::actions::generate_commitment;
use crate::actions::generate_proof;

/// The CLI Args struct.
/// For the description of the params, please refer to the README.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate nullifier, randomness and commitment
    GenerateCommitment,

    /// Generate Proof
    GenerateProof {
        /// Path to serialized Public Parameters file
        #[clap(short, long, value_parser)]
        pp: String,

        /// Leaf index of your commitment
        #[clap(long, value_parser)]
        l: usize,

        /// Root hash among Merkle tree history after your deposit
        #[clap(long, value_parser)]
        root: String,

        /// Merkle tree opening from leaf to root (JSON string or path to JSON file)
        #[clap(long, value_parser)]
        o: String,

        /// Nullifier generated in generate-commitment command
        #[clap(long, value_parser)]
        k: u32,

        /// Randomness generated in generate-commitment command
        #[clap(long, value_parser)]
        r: u32,

        /// Recipient address in SS58
        #[clap(long, value_parser)]
        a: String,

        /// Relayer address in SS58
        #[clap(long, value_parser)]
        t: String,

        /// Relayer fee
        #[clap(long, value_parser)]
        f: u64,

        /// Path to serialized proof file
        #[clap(long, value_parser)]
        output_file: String,
    },
}

impl Commands {
    pub fn do_action(&self) {
        match self {
            Commands::GenerateCommitment => generate_commitment(),
            args @ Commands::GenerateProof { .. } => generate_proof(args),
        }
    }
}
