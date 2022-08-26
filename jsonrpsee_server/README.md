# JSONRPSEE server
Arguments:

- `nullifier_hash` - Ox prefixed hash, e.g. 0x1234 or ASCII data [u8; 32]
- `root` - History stored in the Merkle Tree (0x prefixed hash, e.g. 0x1234 or ASCII data [u8; 32])
- `proof` - Generated proof for transaction (0x prefixed hash, e.g. 0x1234 or ASCII data [u8; 32]) 
- `fee` - Fee (u64)
- `recipient` - Receiver (AccountID)

## Main used libraries:
- [**serde_json**](https://docs.rs/serde_json/1.0.83/serde_json/) : a framework for serializing and deserializing Rust data structures efficiently and generically.
- [**tokio**](https://crates.io/crates/tokio) : an event-driven, non-blocking I/O platform for writing asynchronous applications with the Rust programming language.
- [**subxt**]("https://github.com/paritytech/subxt") :  library to submit extrinsics to a substrate node via RPC. 
- [**jsonrpsee**](https://docs.rs/jsonrpsee/latest/jsonrpsee/) : JSON-RPC protocol library designed for async/await in Rust. 

## Run the server:
To run the server use this command:
```bash
cargo run --release
```

For more information how to use cURL requests in the server: https://www.jsonrpc.org/specification#request_object

Example of input values:

[**HTTP address is generated with the start of the server**] 

- `nullifier_hash` - vpydjyqtbryvuflbjpcuzjtbbthfjymc
- `root` - 0x4ce946e968a0b477960eef24aafe0997350ba8f168ba2e4a546773556bdd1458
- `proof` - Not working now
- `fee` - any number
- `recipient` - 5GcSQPCVXrrWDjPXNnajYDqq24qa92V98cSW9xMzosDDnF3 

cURL usage example:

withdraw
```bash
curl -X POST -H 'Content-Type: application/json' -d '{"jsonrpc":"2.0","id":1,"method":"withdraw","params":["vpydjyqtbryvuflbjpcuzjtbbthfjymc","0x4ce946e968a0b477960eef24aafe0997350ba8f168ba2e4a546773556bdd1458", "10", "5GcSQPCVXrrWDjPXNnajYDqq24qa92V98cSW9xMzosDDnF3u"]}' http://127.0.0.1:51423
```