# Generating proof logic

The library generates proof for the withdrawal method.

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

Circuit checks that the nullifier hash from public inputs equal to the computed nullifier hash from secrets inputs

#### Merkle Tree
Using the leaf index program generates the path from the root to this leaf. For example, for this tree:

```
    R
   / \
  n   o1
 / \
o0  hash(k || r)
```
the path from the root to leaf would be [1, 0], 
where 0 - left, 1 - right,
reverse order due to lifting from leaf to root

After that, using the generated path and tree opening, the circuit computes all hashes on the path. 
In the example above: 
- n hash will be computed using hash(k || r) and o0
- R hash will be computed using n and o1

In the end, the circuit checks that the root hash from public inputs equals the computed root hash

#### Public inputs

Circuit checks that Public inputs, provided for proof generating, are equal to Public inputs provided for proof verification


## WASM:
For a build to wasm:
- install [wasm-pack](https://rustwasm.github.io/wasm-pack/):
`cargo install wasm-pack`
- run this command:
`wasm-pack build`

## Test:

Tests take some time due to proof generating. Recommend running them in release mode with parallel feature:
`cargo test -r --features parallel`  
To run wasm tests:
`wasm-pack test --node -r`