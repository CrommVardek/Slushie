# Main used libraries:
- [**clap**] (https://docs.rs/clap/latest/clap/) for using CLI tools
- [**sp_core**] (https://docs.rs/sp-core/latest/sp_core/) for encoded SS58Check address.
- [**serde_json**] (https://docs.rs/serde_json/1.0.83/serde_json/) for get and read with JSON 

# CLI tool:
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
- `input` - Result of **plonk prover**

Call the **plonk prover** function, with the help of [**clap**] (https://docs.rs/clap/latest/clap/) we enter the initial data and process them with help of [**sp_core**] (https://docs.rs/sp-core/latest/sp_core/) and [**serde_json**] (https://docs.rs/serde_json/1.0.83/serde_json/) for the **plonk prover**