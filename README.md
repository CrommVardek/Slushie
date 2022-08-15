# Slushie

![w3f grants program badge](https://github.com/w3f/Grants-Program/blob/master/src/badge_black.svg)

Slushie is a protocol for private transactions within `pallet-contracts`-compliant networks. It allows users to carry out private balance transfers,
thanks to advanced cryptography.

## Implementation

Slushie is currently implemented as an ink!-based smart-contract, but in the later phases of development, Slushie will also use an off-chain relayer tool
and some more good stuff responsible for generating zero-knowledge proofs.

Slushie uses [`plonk`](https://github.com/dusk-network/plonk) as the ZKP system and [`poseidon252`](https://github.com/dusk-network/Poseidon252) as the
Pedersen hash.

## Running

Build and deploy as a normal `ink!` contract.

## Testing

Test normally with `cargo test`.
Note: the tests may take up to 10 seconds to run.

## Credits

This project was made with :heart_on_fire: by 4IRE with support from Web3 Foundation
