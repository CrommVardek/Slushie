use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short, long, value_parser)]
    pub pp: String,

    #[clap(short, long, value_parser, default_value_t = 1)]
    pub l: usize,

    #[clap(short, long, value_parser)]
    R: String,

    #[clap(short, long, value_parser)]
    o: String,

    #[clap(short, long, value_parser, default_value_t = 1)]
    pub k: u32,

    #[clap(short, long, value_parser, default_value_t = 1)]
    pub r: u32,

    #[clap(short, long, value_parser)]
    pub a: String,

    #[clap(short, long, value_parser)]
    pub t: String,

    #[clap(short, long, value_parser, default_value_t = 1)]
    pub f: u64,

    #[clap(long, value_parser)]
    pub input: String,
}
