use hex;
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use crate::DEFDIP;

pub fn write_to_file(res: &[u8], path: &str) {
    let mut f = File::create(path).expect("Unable to create file");
    f.write_all(res).expect("Unable to write data");
}

pub fn input_json_in_array (json: Value) -> [[u8; 32]; DEFDIP] {
    let mut key_input = [[0; 32]; DEFDIP];
        if let serde_json::Value::Array(arr) = &json {
            for i in 0..DEFDIP {
                if let serde_json::Value::String(json_data) = &arr[i] {
                    key_input[i] = hex::decode(json_data)
                        .unwrap()
                        .try_into()
                        .expect("Unable to write JSON");
                }
            }
        }
        key_input
}

pub fn json_parse(o: &str, file_path: &str) -> [[u8; 32]; DEFDIP] {
    if file_path != "test-json.json" {
        let json: serde_json::Value = serde_json::from_str(o).expect("JSON was not well-formatted");
        input_json_in_array (json)
    } else {
        let file = fs::File::open(file_path).expect("File should open read only");
        let json: serde_json::Value =
            serde_json::from_reader(file).expect("File should be proper JSON");
            input_json_in_array (json)
    }
} 