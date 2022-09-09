use shared::public_types::{PoseidonHash, SerializedProof};

pub struct WithdrawInputs {
    pub nullifier_hash: PoseidonHash,
    pub root: PoseidonHash,
    pub proof: SerializedProof,
    pub fee: u64,
    pub recipient: String,
    pub relayer: String,
}
