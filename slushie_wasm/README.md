# WebAssembly slushie wrapper
Library provides WASM wrapper for [prover library](./plonk_prover/README.md) for future inclusion in the frontend component.

## Available plonk prover functions:

### Generation tree opening

Parameters:
- Uint8Array commitments - flattened array of commitments
- number leaf_index -  leaf index `l`

Function returns flattened tree opening `O(l)`:
Uint8Array opening

### Commitment generation

This function work without parameters.
The function generates and returns an array of nullifier, randomness, commitment, and nullifier hash.

### Proof generating

Parameters:
- Uint8Array pp - serialized public parameters
- number l - leaf index `l`
- Uint8Array R - root hash `R`
- Uint8Array o - flattened tree opening `O(l)`
- number k - nullifier `k`
- number r - randomness `r`
- Uint8Array A - recipient address `A`
- Uint8Array t - relayer address `t`
- bigint f - fee `f`

Function returns serialized proof:
Uint8Array proof

## Build:
For a build to wasm:
- install [wasm-pack](https://rustwasm.github.io/wasm-pack/):
`cargo install wasm-pack`
- run this command:
`wasm-pack build`

## Test:
To run wasm tests:
`wasm-pack test --node -r`

## Note
Tests take a long time due to proof generation.