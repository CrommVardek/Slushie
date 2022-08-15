use clap::Parser;

/// The CLI Args struct.
/// For the description of the params, please refer to the README.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short, long, value_parser)]
    pub pp: String,

    #[clap(long, value_parser, default_value_t = 1)]
    pub l: usize,

    #[clap(long, value_parser)]
    pub rr: String,

    #[clap(long, value_parser)]
    pub o: String,

    #[clap(long, value_parser, default_value_t = 1)]
    pub k: u32,

    #[clap(long, value_parser, default_value_t = 1)]
    pub r: u32,

    #[clap(long, value_parser)]
    pub a: String,

    #[clap(long, value_parser)]
    pub t: String,

    #[clap(long, value_parser, default_value_t = 1)]
    pub f: u64,

    #[clap(long, value_parser)]
    pub output_file: String,
}
