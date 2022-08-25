pub type PoseidonHash = [u8; 32];
pub struct WithdrawInputs {
    pub nullifier_hash: PoseidonHash,
    pub root: PoseidonHash,
    pub proof: [u8; 1040],
    pub fee: u64,
    pub recipient: [u8; 48],
}
