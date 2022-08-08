use sp_core::crypto::*;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub fn get_bytes_from_file(path: &str) -> Vec<u8> {
    let path = Path::new(path);
    let display = path.display();
    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {}: {}", display, why),
        Ok(file) => file,
    };
    let mut bytes = Vec::new();
    if let Err(why) = file.read_to_end(&mut bytes) {
        panic!("Couldn't read {}: {}", display, why)
    };
    bytes
}

pub fn account_id_to_bites(par: &str) -> [u8; 32] {
    match AccountId32::from_ss58check(par) {
        Ok(account_id) => account_id.into(),
        Err(_) => panic!("Not valid account id"),
    }
}

pub fn write_in_file(res: &[u8; 1040], path: &str) {
    let mut f = File::create(path).expect("Unable to create file");
    f.write_all(res).expect("Unable to write data");
}

pub fn json_parce(o: &str) -> Vec<[u8; 32]> {
    let json: serde_json::Value = serde_json::from_str(o).expect("JSON was not well-formatted");
    let arr: [u8; 32] = json.try_into().unwrap();
}
