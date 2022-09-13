use dusk_bytes::Serializable;
pub use dusk_plonk::prelude::Proof;

pub type PoseidonHash = [u8; 32];
pub type SerializedProof = [u8; Proof::SIZE];
pub type Pubkey = [u8; 32];
