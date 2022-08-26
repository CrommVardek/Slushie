# JSONRPSEE server
Arguments:

- `nullifier_hash` - Random 32-bit unsigned numbers generated in CLI
- `root` - The stored in the Merkle tree history
- `proof` - Generated proof for transaction also generated in CLI
- `fee` - Fee
- `recipient` - Receiver

## Main used libraries:
- [**serde_json**](https://docs.rs/serde_json/1.0.83/serde_json/) for JSON manipulations
- [**tokio**](https://crates.io/crates/tokio) a runtime for writing reliable, asynchronous, and slim applications
- [**subxt**]("https://github.com/paritytech/subxt") to call ink! contracts from outside 
- [**jsonrpsee**](https://docs.rs/jsonrpsee/latest/jsonrpsee/) Create and provide for client and server communication over specific protocols

## Run the server:
For running the server you need to use command:
```bash
cargo run --release
```

For more information how to use cUrl requests in the server: https://www.jsonrpc.org/specification#request_object

example of input values:

[**http address generated when server run**] 

- `nullifier_hash` - vpydjyqtbryvuflbjpcuzjtbbthfjymc
- `root` - 0x4ce946e968a0b477960eef24aafe0997350ba8f168ba2e4a546773556bdd1458
- `proof` - Not working now
- `fee` - any number
- `recipient` - 5GcSQPCVXrrWDjPXNnajYDqq24qa92V98cSW9xMzosDDnF3 

example of use cUrl requests:

withdraw
```bash
curl -X POST -H 'Content-Type: application/json' -d '{"jsonrpc":"2.0","id":1,"method":"withdraw","params":["vpydjyqtbryvuflbjpcuzjtbbthfjymc","0x4ce946e968a0b477960eef24aafe0997350ba8f168ba2e4a546773556bdd1458", "10", "5GcSQPCVXrrWDjPXNnajYDqq24qa92V98cSW9xMzosDDnF3u"]}' http://127.0.0.1:51423
```