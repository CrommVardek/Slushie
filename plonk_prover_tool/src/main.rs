mod utils;
use clap::Parser;
use utils::*;
mod commands;
use commands::*;
use plonk_prover::prove;

fn generate_proof(args: Args) {
    let pp = get_bytes_from_file(&args.pp);
    let a = account_id_to_bites(&args.a);
    let t = account_id_to_bites(&args.t);
    match prove(&pp, args.l, [0; 32], vec![], args.k, args.r, a, t, args.f) {
        Ok(result) => write_in_file(&result, &args.input),
        Err(_err) => panic!("Not generated prove"),
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
            R: "test-correct-pp".to_string(),
            o: "test-correct-pp".to_string(),
            l: 1,
            k: 1,
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
            R: "test-correct-pp".to_string(),
            o: "test-correct-pp".to_string(),
            l: 1,
            k: 1,
            r: 1,
            a: "5DtCbNMGwhnP5wJ25Zv59wc5aj5uo3wYdr8536qSRxbvmLdK".to_string(),
            t: "5DtCbNMGwhnP5wJ25Zv59wc5aj5uo3wYdr8536qSRxbvmLdK".to_string(),
            f: 1,
            input: "test-proof".to_string(),
        });
    }
}
