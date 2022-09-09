# Integration tests for Slushie

Integration tests using `polkadot.js` and [`substrate-contracts-node`](https://github.com/paritytech/substrate-contracts-node) for Slushie

## Running the tests

Tests use local [`substrate-contracts-node`](https://github.com/paritytech/substrate-contracts-node), so make sure that it is running. Also, now, deposit and withdraw contracts methods take a lot of weight, so increase `MAXIMUM_BLOCK_WEIGHT` at least to `80 * WEIGHT_PER_SECOND`. In the future, this number should be lower.

From this folder:
Build slushie contract which will be deployed during tests
```bash
cd ../slushie && cargo +nightly contract build --release && cd ../tests 
```
Build slushie wasm wrapper which will be used during tests
```bash
cd ../slushie_wasm && wasm-pack build --release --target nodejs cd ../tests 
```

Install all dependencies and start tests
```
yarn
yarn test
```

## Note
Please note that the tests are *very* long-running. Take your time when running them
