# Generating proof logic

Library which generate proof for withdraw method.

Arguments:
- `pp` - serialized public parameters
- `l` - Leaf index
- `R` - root hash
- `o` - Tree opening (value of sister nodes on the way from leaf l to the root R)
- `k` - Nullifier
- `r` - Randomness
- `A` - Recipient address
- `t` - Relayer address
- `f` - Fee

## Main used libraries:

- [`dusk-plonk`](https://github.com/dusk-network/plonk) - rust implementation of the PLONK ZKProof System
- [`dusk-poseidon`](https://github.com/dusk-network/poseidon252) - implementation for the Poseidon Snark-friendly Hash algorithm
- [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen) - crate for interactions between Wasm modules and JavaScript

## Circuit checks:

#### Nullifier Hash

Circuit checks that nullifier hash from public inputs equal to the computed nullifier hash from secrets inputs

#### Merkle Tree

Using leaf index program generate path from root to this leaf. For example, for this tree:

```
    R
   / \
  n   o1
 / \
o0  hash(k || r)
```
path from root to leaf would be [1, 0], 
where 0 - left, 1 - right,
reverse order due to lifting from leaf to root

After that, using generated path and tree opening, circuit computes all hashes on the path. 

In example above: 
- n hash will be computed using hash(k || r) and o0
- R hash will be computed using n and o1

In the end, circuit checks that root hash from public inputs equal to the computed root hash

#### Public inputs

Circuit checks that Public inputs provided for proof generating equals Public inputs provided for proof verification


## WASM:

For build to wasm:
- install [wasm-pack](https://rustwasm.github.io/wasm-pack/):
`cargo install wasm-pack`
- run this command:
`wasm-pack build`

## Test:

Tests take some time due to proof generating. Recommend running them in release mode with parallel feature:
`cargo test -r --features parallel`  
To run wasm tests:
`wasm-pack test --node -r`