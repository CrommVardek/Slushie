mod utils;
use std::{fs::File, io::Read, path::Path};

use clap::Parser;
use sp_core::crypto::{AccountId32, Ss58Codec};
use utils::*;
mod commands;
use commands::*;
use hex;
use plonk_prover::prove;

//size of array that contains keys
pub const DEPTH: usize = 2;

fn generate_proof(args: Args) {
    let path = Path::new(&args.pp);
    let mut bytes = Vec::new();
    let _ = File::open(&path).unwrap().read_to_end(&mut bytes);
    let a = AccountId32::from_ss58check(&args.a)
        .expect("Could not convert input to AccountId32")
        .into();
    let t = AccountId32::from_ss58check(&args.t)
        .expect("Could not convert input to AccountId32")
        .into();
    let o = json_parse(&args.o);
    let rr: [u8; 32] = hex::decode(args.rr).unwrap().try_into().unwrap();
    let proof = prove(&bytes, args.l, rr, o, args.k, args.r, a, t, args.f);
    write_to_file(&proof.expect("Could not generate proof"), &args.input);
}
fn main() {
    generate_proof(Args::parse());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn key_generated() {
        generate_proof(Args {
            pp: "test-correct-pp".to_string(),
            rr: "28f396386a802cf7ffdc2288c0a95c5807730c504bb8ba8b6c1033c598ddf4dd".to_string(),
            o: r#"["1422626DF22F8FDC85D3F1B54B05DAE703D545326D957C05089191C39D34CB74","1422626DF22F8FDC85D3F1B54B05DAE703D545326D957C05089191C39D34CB74"]"#.to_string(),
            l: 1,
            k: 3141592653,
            r: 1,
            a: "5DtCbNMGwhnP5wJ25Zv59wc5aj5uo3wYdr8536qSRxbvmLdK".to_string(),
            t: "5DtCbNMGwhnP5wJ25Zv59wc5aj5uo3wYdr8536qSRxbvmLdK".to_string(),
            f: 1,
            input: "test-proof".to_string(),
        });
    }

    #[test]
    fn key_generated_file_json() {
        generate_proof(Args {
            pp: "test-correct-pp".to_string(),
            rr: "28f396386a802cf7ffdc2288c0a95c5807730c504bb8ba8b6c1033c598ddf4dd".to_string(),
            o: "test-json.json".to_string(),
            l: 1,
            k: 3141592653,
            r: 1,
            a: "5DtCbNMGwhnP5wJ25Zv59wc5aj5uo3wYdr8536qSRxbvmLdK".to_string(),
            t: "5DtCbNMGwhnP5wJ25Zv59wc5aj5uo3wYdr8536qSRxbvmLdK".to_string(),
            f: 1,
            input: "test-proof".to_string(),
        });
    }

    #[test]
    #[should_panic]
    fn key_not_generated() {
        generate_proof(Args {
            pp: "test-wrong-pp".to_string(),
            rr: "28f396386a802cf7ffdc2288c0a95c5807730c504bb8ba8b6c1033c598ddf4dd".to_string(),
            o: r#"["1422626DF22F8FDC85D3F1B54B05DAE703D545326D957C05089191C39D34CB74","1422626DF22F8FDC85D3F1B54B05DAE703D545326D957C05089191C39D34CB74"]"#.to_string(),
            l: 1,
            k: 3141592653,
            r: 1,
            a: "5DtCbNMGwhnP5wJ25Zv59wc5aj5uo3wYdr8536qSRxbvmLdK".to_string(),
            t: "5DtCbNMGwhnP5wJ25Zv59wc5aj5uo3wYdr8536qSRxbvmLdK".to_string(),
            f: 1,
            input: "test-proof".to_string(),
        });
    }
}
