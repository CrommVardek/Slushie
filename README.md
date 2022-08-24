# Slushie

![w3f grants program badge](https://github.com/w3f/Grants-Program/blob/master/src/badge_black.svg)

Slushie is a protocol for private transactions within `pallet-contracts`-compliant networks. It allows users to carry out private balance transfers,
thanks to advanced cryptography.

## General description

Slushie has the following functionality:

- Deposit money to the contract. This can be done in a single transaction with a fixed
amount (denoted by `N`), which has been set up during initialization (`deposit size`)
- Withdraw money from the contract. The `N` tokens are withdrawn through a Relayer with sending fee (denoted by `f`) tokens as a fee to the Relayer address (denoted by `t`) and `N - f` tokens to the recipient address (denoted by `A`). The withdrawal transaction is initiated by the Relayer and it pays the transaction fee that is covered by `f`.

### Initialization

During initialization, the contract accepts `N` as a parameter. Also, the contract creates a Merkle tree of height 20.

Merkle tree features:
 - each non-leaf node hashes its 2 children with Poseidon Hash (denoted by `H`)
 - initialized with all zero leaves where each equals `blake2x256("slushie")`
 - stores the last 100 root values in the history array. 
 - for the latest Merkle tree, stores the values of nodes on the path from the last added leaf to the root that is necessary to compute the next root.

### Deposit

To deposit, a user:
1. Generate two random 32-bit unsigned numbers nullifier (denoted by `k`), randomness (denoted by `r`), and computes commitment (denoted by `C`) such that `C = H(k || n)` 
2. Send transaction with `N` tokens to contract with data `C` interpreted as 32 bytes array.

If the tree is not full, the contract accepts the transaction, inserts `C` into the tree as a new non-zero leaf and recalculates the path from the last added value and the latest root. The previous root is added to the history array. Also, the contract emits a "Deposited" event, which includes `C` that will be used for finding the leaf index of `C` (denoted by `l`), computing Merkle opening (value of sister nodes on the way from leaf `l` to the root `R`, denoted by `O(l)`) and Merkle path (path from `R` to `l`, denoted by `p(l)`).

### Withdrawal

To withdraw a user:
1. Select an `A` and `f` value such that `f ≤ N`
2. Select an `R` among the stored ones in the Merkle tree history and compute `O(l)`, `p(l)` 
3. Compute nullifier hash (denoted by `h`) `h = H(k)`
4. Generate proof (denoted by `P`)
5. Send a request to Relayer supplying transaction data `R`, `h`, `A`, `f`, `t`, `P`. Then the Relayer makes a Withdrawal transaction to contract with supplied data.

The contract verifies the proof and uniqueness of the nullifier hash to guarantee that proof has not appeared before. If verification successes, it sends `N − f` to `A` and `f` to the `t` and adds `h` to the list of nullifier hashes.

### Zero-knowledge proof scheme

For proof generating and verification, Slushie uses the zero-knowledge proof scheme called PLONK. 

PLONK circuit has such inputs:
Private:
- `k`
- `r`
- `O(l)`
- `p(l)`

Public:
- `R`
- `h`
- `A`
- `t`
- `f`

`A`, `t`, `f` are included in the circuit to guarantee that provided inputs for generating proof equal to provided inputs for verification.

Also for generating proof and verification PLONK uses Public Parameters (denoted by `pp`) which later will be generated during the trusted setup ceremony, but for now, it is hardcoded in the file.

#### Proof generation 

Proof generation function use `pp`, `l`, `R`, `O(l)`, `k`, `r`, `A`, `t`, `f`. It computes `p(l)` using `l` and then using `pp`, Public and Private inputs, generates a proof and serialized it.

In general, the circuit has such main constraints:

- `A`, `t`, `f` are the same for generating and verifying
- calculated in circuit `H(k)`, which for calculation used provided secret `k`, equals to public `h`
- calculated in circuit `R`, which for calculation used provided secret `p(l)`, `O(l)`, `k`, `r`, equals to public `R`

#### Proof verification (in progress)

Proof verification function will use `pp`, `R`, `A`, `t`, `f`. Using `pp` and Public inputs, verify proof and then return `true` in a successful case, otherwise, return `false`.

## Implementation

Slushie is currently implemented as an [ink!-based smart-contract](./slushie/usage.md), [prover library](./plonk_prover/README.md), and the [CLI tool](./plonk_prover_tool/README.md) to generate proofs in the off-chain context.

Slushie uses [`plonk`](https://github.com/dusk-network/plonk) as the ZKP system and [`poseidon252`](https://github.com/dusk-network/Poseidon252) as the
Pedersen hash.

## Running

Build and deploy as a normal `ink!` contract.

## Testing

Test normally with `cargo test`.
Note: the tests may take up to 10 seconds to run.

## Credits

This project was made with :heart_on_fire: by 4IRE with support from Web3 Foundation
