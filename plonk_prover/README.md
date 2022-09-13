# Zero-Knowledge Proofs part of Slushie
The library for manipulating with [`dusk-plonk`](https://github.com/dusk-network/plonk). To be used in the contract and frontend component.

## Main functions:

### Proof generation
This function generates serialized plonk proof which will be used in withdraw contract method to verify knowledge of the randomness & nullifier.

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

#### JS Compatibility
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

### Proof verification

The library provides the two verifying techniques:
- Verification with Public Parameters. This method supposes having access to the Public Parameters. In this way, it can be used with different tree depths due to the inner compiling circuit for certain depths. However, slower and use space for Public Parameters (~3MiB) than the next.
- Verification without Public Parameters. This method uses precompiled verifier data and the opening key for default Slushie tree depth. Faster and use less space due to using only verifier data(vd-test) and the opening key(op-key-test) (~1.2KiB)

#### Verification with Public Parameters:
This method is used in [ink!-based smart-contract](./slushie/usage.md) for increasing performance 
Arguments:
- `pp` - Serialized public parameters
- `R` - Root hash
- `A` - Recipient address
- `t` - Relayer address
- `f` - Fee
- `P` - Generated serialized proof

#### Verification without Public Parameters:
Arguments:
- `vd` - Serialized verifier data
- `opening key` - Opening key
- `R` - Root hash
- `A` - Recipient address
- `t` - Relayer address
- `f` - Fee
- `P` - Generated serialized proof

### Generation tree opening

Library provides the interface for creating a Merkle tree from a slice of commitments and generating tree opening using it.

#### JS Compatibility

Parameters:
- Uint8Array commitments - flattened array of commitments
- number leaf_index -  leaf index `l`

Function returns flattened tree opening `O(l)`:
Uint8Array opening

### Commitment generation

This function generates two random 32-bit unsigned numbers nullifier `k`, randomness `r`, and then computes commitment `C` such that `C = H(k || r)` and nullifier hash `k` such that `h = H(k)`.  This command work without parameters.

#### JS Compatibility
The function returns an array of `k`, `r`, `c`, `h`:
[number k, number r, Uint8Array c, Uint8Array h]

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

## WASM Build:
For a build to wasm:
- install [wasm-pack](https://rustwasm.github.io/wasm-pack/):
`cargo install wasm-pack`
- run this command:
`wasm-pack build --features js`

## Test:

Tests take some time due to proof generation. Recommend running them in release mode with parallel feature:
`cargo test -r --features parallel`  

To run wasm tests:
`wasm-pack test --node -r --features js`