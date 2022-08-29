# WebAssembly slushie wrapper
Library provides WASM wrapper for [prover library](./plonk_prover/README.md) for future inclusion in the frontend component.

## Available plonk prover functions:

### Proof generating

Parameters:
- Uint8Array pp - Serialized public parameters
- number l - Leaf index `l`
- Uint8Array R - Root hash `R`
- Uint8Array o - Flatten tree opening `O(l)`
- number k - Nullifier `k`
- number r - Randomness `r`
- Uint8Array A - Recipient address `A`
- Uint8Array t - Relayer address `t`
- bigint f - Fee `f`

Function returns serialized proof:
Uint8Array proof

## Building:
For a build to wasm:
- install [wasm-pack](https://rustwasm.github.io/wasm-pack/):
`cargo install wasm-pack`
- run this command:
`wasm-pack build`

## Test:
To run wasm tests:
`wasm-pack test --node -r`

## Note
Tests take some time due to proof generating.