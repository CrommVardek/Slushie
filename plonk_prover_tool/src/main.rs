mod utils;
use std::io::Write;
use std::{fs::File, io::Read, path::Path};

use clap::Parser;
use sp_core::crypto::{AccountId32, Ss58Codec};
use utils::*;
mod commands;
use commands::*;
use plonk_prover::prove;

//size of array that contains keys
use shared::constants::DEFAULT_DEPTH;

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
    let o = parse_tree_openings(&args.o);
    let rr: [u8; 32] = hex::decode(args.rr).unwrap().try_into().unwrap();
    let proof =
        prove(&bytes, args.l, rr, o, args.k, args.r, a, t, args.f).expect("Error generating proof");
    let mut output_file = File::create(&args.output_file).expect("Unable to create file");
    output_file
        .write_all(&proof)
        .expect("Unable to write proof to file");
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
            rr: "0EDB120C1F24145A221C3B77D15ABC9959956FBE7E3B37832166CCB7ADE0CFCD".to_string(),
            o: r#"[           
                "8B275561AB29C3EE4FDFF5AA9912AAC193B3A76F474B2C7B0AC60C0B69841AE2",
                "AE603307AABE14B1685F623B22313F954277762A377CA4D64DBAFEC8F06DBECE",
                "6BB536DE520D253D0A2C54FE5F186AA799527E29DA190A88234E4A46F7040829",
                "79DD7DDF25DAD6F2BF0E15EA31C7CC3C1E9DF8AD7602E1A41987C41823BA5FC0",
                "F8BCF2B724183F97F9231540F6BED2180B010F57DB38E7356BAEBE5FA7FE0592",
                "B9ACD7A16684D97DC0640F27847861635A6AD6EC19737640096931ED7EFCF023",
                "69FE3804090C7EB6A3DC26910406328E88AD9A1A3D17A21A488C0782D3CD1ADB",
                "82E8D10EA7372F064EF182A7A582F8E4C75600AA5C6BED5C1ACFB46F3117B8BA",
                "11FD66237D4AE27A0C9E254317DCACE75A1D0A58028C3F24019F41A23125AB9E",
                "FE44E37502DA6662F83BF0A1AC0D87C5C6E48D5E729FD7DB517F3F9199A67AE9",
                "51AE78857AD8DF490AE44880EA4C04E8B07F239F3FA6997F4C7FEFAE569E2E2D",
                "0E741D218670C466CB8A6B1AE27302118FE3B18FB0E0815C2997E1EB96E85410",
                "CBBB12B173FDF0EB4F29ACD9AD077F76E507AC01F7A218C80CAD31C702152499",
                "42B6A37B934DF6CB4DC28C70ADC862ECA37A0C373E2FB39E35D9DC33437529F6",
                "BAABDB3671917F9F7A95D1B6ACD0498911AFEF04EBD777953CB959BB8BDB2C1A",
                "5345BF5B8DEDC82AF77B9C1E7479C850C2EE8980C3FC914848B10DD8CC2E09BD",
                "BA3859D1ED676D0E4D8216CDBC0DD41058CFADCEBA6ABCB2691EF0EB48EA06E1",
                "12FD1D45C71ACEB8AE52058263A7654A420347B75F098F1F6FC8F1266DEE082E",
                "E6ECF20DEBA92629846283A3BF257D9E473422781184902D6BD7288B5A4EBD41",
                "9ED838A9F6CB58FE464FA06DD5905D58AEDD9E4BD30ED0840E4828B57C5B7324"
                ]"#
            .to_string(),
            l: 1,
            k: 3141592653,
            r: 1,
            a: "5DtCbNMGwhnP5wJ25Zv59wc5aj5uo3wYdr8536qSRxbvmLdK".to_string(),
            t: "5DtCbNMGwhnP5wJ25Zv59wc5aj5uo3wYdr8536qSRxbvmLdK".to_string(),
            f: 1,
            output_file: "test-proof".to_string(),
        });
    }

    #[test]
    fn key_generated_file_json() {
        generate_proof(Args {
            pp: "test-correct-pp".to_string(),
            rr: "0EDB120C1F24145A221C3B77D15ABC9959956FBE7E3B37832166CCB7ADE0CFCD".to_string(),
            o: "test-json.json".to_string(),
            l: 1,
            k: 3141592653,
            r: 1,
            a: "5DtCbNMGwhnP5wJ25Zv59wc5aj5uo3wYdr8536qSRxbvmLdK".to_string(),
            t: "5DtCbNMGwhnP5wJ25Zv59wc5aj5uo3wYdr8536qSRxbvmLdK".to_string(),
            f: 1,
            output_file: "test-proof".to_string(),
        });
    }

    #[test]
    #[should_panic]
    fn key_not_generated() {
        generate_proof(Args {
            pp: "test-wrong-pp".to_string(),
            rr: "0EDB120C1F24145A221C3B77D15ABC9959956FBE7E3B37832166CCB7ADE0CFCD".to_string(),
            o: r#"[           
                "8B275561AB29C3EE4FDFF5AA9912AAC193B3A76F474B2C7B0AC60C0B69841AE2",
                "AE603307AABE14B1685F623B22313F954277762A377CA4D64DBAFEC8F06DBECE",
                "6BB536DE520D253D0A2C54FE5F186AA799527E29DA190A88234E4A46F7040829",
                "79DD7DDF25DAD6F2BF0E15EA31C7CC3C1E9DF8AD7602E1A41987C41823BA5FC0",
                "F8BCF2B724183F97F9231540F6BED2180B010F57DB38E7356BAEBE5FA7FE0592",
                "B9ACD7A16684D97DC0640F27847861635A6AD6EC19737640096931ED7EFCF023",
                "69FE3804090C7EB6A3DC26910406328E88AD9A1A3D17A21A488C0782D3CD1ADB",
                "82E8D10EA7372F064EF182A7A582F8E4C75600AA5C6BED5C1ACFB46F3117B8BA",
                "11FD66237D4AE27A0C9E254317DCACE75A1D0A58028C3F24019F41A23125AB9E",
                "FE44E37502DA6662F83BF0A1AC0D87C5C6E48D5E729FD7DB517F3F9199A67AE9",
                "51AE78857AD8DF490AE44880EA4C04E8B07F239F3FA6997F4C7FEFAE569E2E2D",
                "0E741D218670C466CB8A6B1AE27302118FE3B18FB0E0815C2997E1EB96E85410",
                "CBBB12B173FDF0EB4F29ACD9AD077F76E507AC01F7A218C80CAD31C702152499",
                "42B6A37B934DF6CB4DC28C70ADC862ECA37A0C373E2FB39E35D9DC33437529F6",
                "BAABDB3671917F9F7A95D1B6ACD0498911AFEF04EBD777953CB959BB8BDB2C1A",
                "5345BF5B8DEDC82AF77B9C1E7479C850C2EE8980C3FC914848B10DD8CC2E09BD",
                "BA3859D1ED676D0E4D8216CDBC0DD41058CFADCEBA6ABCB2691EF0EB48EA06E1",
                "12FD1D45C71ACEB8AE52058263A7654A420347B75F098F1F6FC8F1266DEE082E",
                "E6ECF20DEBA92629846283A3BF257D9E473422781184902D6BD7288B5A4EBD41",
                "9ED838A9F6CB58FE464FA06DD5905D58AEDD9E4BD30ED0840E4828B57C5B7324"
                ]"#
            .to_string(),
            l: 1,
            k: 3141592653,
            r: 1,
            a: "5DtCbNMGwhnP5wJ25Zv59wc5aj5uo3wYdr8536qSRxbvmLdK".to_string(),
            t: "5DtCbNMGwhnP5wJ25Zv59wc5aj5uo3wYdr8536qSRxbvmLdK".to_string(),
            f: 1,
            output_file: "test-proof".to_string(),
        });
    }
}