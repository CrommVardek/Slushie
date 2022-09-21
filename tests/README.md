# Integration tests for Slushie

Integration tests using `polkadot.js` and [`substrate-contracts-node`](https://github.com/paritytech/substrate-contracts-node) for Slushie

## Running the tests

Tests use local [`substrate-contracts-node`](https://github.com/paritytech/substrate-contracts-node), so make sure that it is running. Also, now, deposit and withdraw contracts methods take a lot of weight, so increase `MAXIMUM_BLOCK_WEIGHT` at least to `80 * WEIGHT_PER_SECOND`. In the future, this number should be lower.

From this folder:
Generate prover data:
```bash
cd ../plonk_prover_tool && cargo run -r -- generate-prover-data --pp ../public-parameters/pp-test --output-pd ../public-parameters/pd-test --output-ck  ../public-parameters/commit-key-test && cd ../tests 
```

Build slushie contract which will be deployed during tests
```bash
cd ../slushie && cargo +nightly contract build --release && cd ../tests 
```
Build slushie wasm wrapper which will be used during tests (we use the js_include_pd feature to speed up testing)
```bash
cd ../plonk_prover && wasm-pack build --target nodejs --features js_include_pd && cd ../tests 
```

Install all dependencies and start tests
```
yarn
yarn test
```

## Note
Please note that the tests are *very* long-running. Take your time when running them
