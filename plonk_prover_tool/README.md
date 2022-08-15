# The Plonk Prover CLI tool
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
- `output-file` - where to output the proof

## Main used libraries:
- [**clap**](https://docs.rs/clap/latest/clap/) for the core CLI logic
- [**sp_core**](https://docs.rs/sp-core/latest/sp_core/) for decoding ss58-encoded addresses
- [**serde_json**](https://docs.rs/serde_json/1.0.83/serde_json/) for JSON manipulations


### Note
Please note that the tests are *very* long-running. Take your time when running them :)