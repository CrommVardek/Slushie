# The Plonk Prover CLI tool

This CLI tool can be used as the helper for the Slushie mixer. This tool in cooperation with [polkadot.js](https://polkadot.js.org/) should provide access to all Slushie features.

## General functionality
Slushie CLI has the following functionality:

- Generate commitment. This command generates random 32-bit numbers randomness `r` and nullifier `k`. Also, using these values, commitment `C` and nullifier hash `h` are computed. All these values are important for Slushie's correct work. 
After generating you can use these values in such ways:
  - `C` as a parameter of the deposit contracts method
  - `r`, `k`, `h` as parameters of the generating proof command
  - `h` as a parameter of the withdraw contracts method

- Get leaf index `l`. Using `C`, this command returns leaf index `l`, which will be used in proof generating. (Command in progress) 

- Get root `R` for `l`. Using `l`, this command returns suitable root `R` for provided `l`, which will be used in proof generating. (Command in progress) 

- Generate Merkle opening `O(l)` for `l`. This command returns Merkle tree opening for provided `l`. This value will be used in proof generating. (Command in progress) 

- Generate proof. Using `l`, `R`, `O(l)`, `k`, `r`, this command generates serialized plonk proof which will be used in withdraw contract method to verify knowledge of the randomness & nullifier.

## Commands in details

### Commitment generation 

This command generates two random 32-bit unsigned numbers nullifier `k`, randomness `r`, and then computes commitment `C` such that `C = H(k || r)` and nullifier hash `k` such that `h = H(k)`. After that, print all these values. This command work without parameters.

Example of running this command:

```bash
cargo run -r -- generate-commitment
```

### Proof generation
For generating proof this tool uses these arguments:

- `pp` - Path to file with serialized Public Parameters `pp`, which are hardcoded for now in the `test-correct-pp` file and later will be generated from a trusted setup ceremony
- `l` - Leaf index of commitment `C` as a number, generated in the get leaf index command 
- `root` - 32 bytes of Root `R` in hex format, generated in the get root command 
- `o` - Path to JSON file with 32 bytes default size array of Merkle opening `O(l)` in hex format or JSON string with the same contents, generated in Merkle opening generation command 
- `k` - Nullifier `k` - random 32-bit unsigned integer, generated in commitment generation command
- `r` - Randomness `r` - random 32-bit unsigned integer, generated in commitment generation command
- `a` - Recipient address `A` in SS58 on which contract will send `N - fee` Tokens, where N is deposit size and has been set up during contract initialization and fee - Relayer fee
- `t` - Relayer address `t` in SS58 on which contract will send the `fee`
- `f` - Relayer fee `f`
- `output-file` - Path to generated file with serialized proof

Example of running this command:

```bash
cargo run -r  -- generate-proof --pp ../public-parameters/pp-test --l 1 --root 0EDB120C1F24145A221C3B77D15ABC9959956FBE7E3B37832166CCB7ADE0CFCD --o test-json.json --k 3141592653 --r 1 --a 5DtCbNMGwhnP5wJ25Zv59wc5aj5uo3wYdr8536qSRxbvmLdK --t 5DtCbNMGwhnP5wJ25Zv59wc5aj5uo3wYdr8536qSRxbvmLdK --f 1 --output-file test-proof
```

### Public parameters generation

For generating Public parameters this tool uses these arguments:

- `output-pp` - Path to output file with serialized Public Parameters `pp`, which are hardcoded for now in the `test-correct-pp` file and later will be generated from a trusted setup ceremony

```bash
 cargo run -r -- generate-test-public-parameters --output-pp ./pp 
```

### Verifier data generation

For generating verifier data this tool uses these arguments:

- `pp` - Path to file with serialized Public Parameters `pp`, which are hardcoded for now in the `test-correct-pp` file and later will be generated from a trusted setup ceremony
- `output-vd` - Path to output file with serialized verifier data
- `output-ok` - Path to output file with serialized opening key

```bash
cargo run -r -- generate-verifier-data --pp ../public-parameters/pp-test --output-vd ../public-parameters/vd-test --output-ok  ../public-parameters/opening-key-test
```

### Prover data generation

For generating prover data this tool uses these arguments:

- `pp` - Path to file with serialized Public Parameters `pp`, which are hardcoded for now in the `test-correct-pp` file and later will be generated from a trusted setup ceremony
- `output-pd` - Path to output file with serialized prover data
- `output-ck` - Path to output file with serialized commit key

```bash
cargo run -r -- generate-prover-data --pp ../public-parameters/pp-test --output-pd ../public-parameters/pd-test --output-ck  ../public-parameters/commit-key-test
```


### Get leaf index

Command in progress

### Get root

Command in progress

### Generate Merkle opening

Command in progress

## Main used libraries:
- [**clap**](https://docs.rs/clap/latest/clap/) for the core CLI logic
- [**sp_core**](https://docs.rs/sp-core/latest/sp_core/) for decoding ss58-encoded addresses
- [**serde_json**](https://docs.rs/serde_json/1.0.83/serde_json/) for JSON manipulations

### Note
Please note that the tests are *very* long-running. Take your time when running them :)
## Run CLI:
for running CLI you need to use the command:
```bash
cargo run -r -- command --name value 
```
- name - the name of the argument
- value - value that you want to input for CLI
- command - command which you want to run