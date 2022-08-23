# The Plonk Prover CLI tool
Arguments:

- `pp` - serialized public parameters
- `l` - Leaf index
- `rr` - root hash
- `o` - Tree opening (value of sister nodes on the way from leaf l to the root R)
- `k` - Nullifier
- `r` - Randomness
- `a` - Recipient address
- `t` - Relayer address
- `f` - Fee
- `output-file` - where to output the proof

## Main used libraries:
- [**clap**](https://docs.rs/clap/latest/clap/) for the core CLI logic
- [**sp_core**](https://docs.rs/sp-core/latest/sp_core/) for decoding ss58-encoded addresses
- [**serde_json**](https://docs.rs/serde_json/1.0.83/serde_json/) for JSON manipulations

### Note
Please note that the tests are *very* long-running. Take your time when running them :)
## Run CLI:
for running CLI you need to use command:
```bash
cargo run -r -- --name value 
```
- name - name of arguments
- value - value that you want to input for CLI

## Example of input values:
- `pp` - test-correct-pp
- `l` - 1
- `rr` - 0EDB120C1F24145A221C3B77D15ABC9959956FBE7E3B37832166CCB7ADE0CFCD
- `o` - test-json.json
- `k` - 3141592653
- `r` - 1
- `a` - 5DtCbNMGwhnP5wJ25Zv59wc5aj5uo3wYdr8536qSRxbvmLdK
- `t` - 5DtCbNMGwhnP5wJ25Zv59wc5aj5uo3wYdr8536qSRxbvmLdK
- `f` - 1
- `output-file` - "output file name"

```bash
cargo run -r  -- --pp test-correct-pp --l 1 --rr 0EDB120C1F24145A221C3B77D15ABC9959956FBE7E3B37832166CCB7ADE0CFCD --o test-json.json --k 3141592653 --r 1 --a 5DtCbNMGwhnP5wJ25Zv59wc5aj5uo3wYdr8536qSRxbvmLdK --t 5DtCbNMGwhnP5wJ25Zv59wc5aj5uo3wYdr8536qSRxbvmLdK --f 1 --output-file test-proof
```