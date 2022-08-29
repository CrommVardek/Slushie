use crate::slushie::PublicInputs;
use ink_env::AccountId;

use plonk_prover::verify_with_vd;

pub(crate) fn check_proof(
    vd: &[u8],
    opening_key: &[u8; 240],
    public_inputs: &PublicInputs,
    relayer: AccountId,
) -> bool {
    verify_with_vd(
        vd,
        opening_key,
        public_inputs.nullifier_hash,
        public_inputs.root,
        *public_inputs.recipient.as_ref(),
        *relayer.as_ref(),
        public_inputs.fee,
        &public_inputs.proof,
    )
    .map(|_| true)
    .unwrap_or(false)
}
