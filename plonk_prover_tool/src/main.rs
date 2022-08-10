mod utils;
use clap::Parser;
use utils::*;
mod commands;
use commands::*;
use hex;
use plonk_prover::prove;

pub const DEFDIP: usize = 2;

fn generate_proof(args: Args) {
    let pp = include_bytes(&args.pp);
    let a = account_id_to_bites(&args.a);
    let t = account_id_to_bites(&args.t);
    let o = json_parce(&args.o, &args.switch_to_file);
    let rr: [u8; 32] = hex::decode(args.rr).unwrap().try_into().unwrap();
    match prove(&pp, args.l, rr, o, args.k, args.r, a, t, args.f) {
        Ok(result) => write_in_file(&result, &args.input),
        Err(_err) => panic!("{} Not generated prove", _err),
    }
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
            switch_to_file: "fil".to_string(),
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
            rr: "test-correct-pp".to_string(),
            o: "test-correct-pp".to_string(),
            switch_to_file: "file".to_string(),
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
