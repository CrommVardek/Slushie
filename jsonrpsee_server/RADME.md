# JSONRPSEE server
Arguments:

- `nullifier_hash` - random 32-bit unsigned numbers generated in CLI
- `root` - 32 bytes ///cli
- `proof` - Generated proof for transaction also generated in CLI
- `fee` - Fee
- `recipient` - Receiver

## Main used libraries:
- [**shared**](https://crates.io/crates/shared) a macro for safely sharing data between application and interrupt context on cortex-m systems
- [**serde_json**](https://docs.rs/serde_json/1.0.83/serde_json/) for JSON manipulations
- [**tokio**](https://crates.io/crates/tokio) a runtime for writing reliable, asynchronous, and slim applications
- [**subxt**]("https://github.com/paritytech/subxt") for call the ink! contract's function from the outside
- [**jsonrpsee**](https://docs.rs/jsonrpsee/latest/jsonrpsee/) for create and provide for client and server communication over specific protocols

## Run JSONRPSEE:
For running jsonrpsee you need to use command:
```bash
cargo run --release
```

For more information how to use cUrl requests in jsonrpsee: https://www.jsonrpc.org/specification#request_object

example of input values:

[**http address generated when server run**]

- `nullifier_hash` - vpydjyqtbryvuflbjpcuzjtbbthfjymc
- `root` - 0x4ce946e968a0b477960eef24aafe0997350ba8f168ba2e4a546773556bdd1458
- `proof` - Not working now
- `fee` - any number that you want
- `recipient` - 5GcSQPCVXrrWDjPXNnajYDqq24qa92V98cSW9xMzosDDnF3

example of use cUrl requests:

withdraw
```bash
curl -X POST -H 'Content-Type: application/json' -d '{"jsonrpc":"2.0","id":1,"method":"withdraw","params":["vpydjyqtbryvuflbjpcuzjtbbthfjymc","0x4ce946e968a0b477960eef24aafe0997350ba8f168ba2e4a546773556bdd1458", "10", "5GcSQPCVXrrWDjPXNnajYDqq24qa92V98cSW9xMzosDDnF3u"]}' http://127.0.0.1:51423
```