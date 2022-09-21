use crate::DEFAULT_DEPTH;
use serde_json::Value;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

/// This function parses the tree openings, formatted as a JSON array and encoded in hex.
pub fn parse_tree_openings(o: &str) -> [[u8; 32]; DEFAULT_DEPTH] {
    let convert_json_array_to_bytes = |json: Value| {
        let mut key_input = [[0; 32]; DEFAULT_DEPTH];
        if let Value::Array(arr) = &json {
            for i in 0..DEFAULT_DEPTH {
                if let Value::String(json_data) = &arr[i] {
                    key_input[i] = hex::decode(json_data)
                        .unwrap()
                        .try_into()
                        .expect("Unable to write JSON");
                }
            }
        }
        key_input
    };

    // check if we're provided a file
    if !o.ends_with("json") {
        let json: Value = serde_json::from_str(o).expect("JSON was not well-formatted");
        convert_json_array_to_bytes(json)
    } else {
        let file = File::open(o).expect("File should open read only");
        let json: Value = serde_json::from_reader(file).expect("File should be proper JSON");
        convert_json_array_to_bytes(json)
    }
}

/// Read public parameters from file
pub fn read_pp(path: &str) -> Vec<u8> {
    let path = Path::new(path);

    let mut pp_bytes = Vec::new();

    File::open(path)
        .unwrap()
        .read_to_end(&mut pp_bytes)
        .expect("Unable to read Public Parameters from file");

    pp_bytes
}

pub fn write_to_file(output_file: &str, content: &[u8]) {
    let mut output_file = File::create(output_file).expect("Unable to create file");
    output_file
        .write_all(content)
        .expect("Unable to write proof to file");
}
