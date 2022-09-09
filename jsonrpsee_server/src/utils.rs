use jsonrpsee::types::error::CallError;
use plonk_prover::{verify, Pubkey};
use shared::constants::DEFAULT_DEPTH;
use sp_core::crypto::{AccountId32, Ss58Codec};

use crate::public_inputs::WithdrawInputs;

/// Proof verification.
pub async fn verify_proof(inputs: &WithdrawInputs) -> Result<(), Box<dyn std::error::Error>> {
    let public_parameters = include_bytes!("test-correct-pp");

    let recipient: Pubkey = AccountId32::from_ss58check(&inputs.recipient)
        .map_err(|_| {
            CallError::InvalidParams(anyhow::Error::msg("Cannot decode recipient parameter."))
        })?
        .try_into()
        .map_err(|_| {
            CallError::InvalidParams(anyhow::Error::msg("Invalid recipient parameter."))
        })?;

    let relayer: Pubkey = AccountId32::from_ss58check(&inputs.relayer)
        .map_err(|_| {
            CallError::InvalidParams(anyhow::Error::msg("Cannot decode relayer parameter."))
        })?
        .try_into()
        .map_err(|_| CallError::InvalidParams(anyhow::Error::msg("Invalid relayer parameter.")))?;

    verify::<{ DEFAULT_DEPTH }>(
        public_parameters,
        inputs.nullifier_hash,
        inputs.root,
        recipient,
        relayer,
        inputs.fee,
        &inputs.proof,
    )
    .map_err(|_| CallError::InvalidParams(anyhow::Error::msg("Invalid proof.")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{public_inputs::WithdrawInputs, utils::verify_proof};
    use subxt::ext::sp_core::bytes::from_hex;

    #[tokio::test]
    async fn test_proof_verification_correct() {
        let proof = hex::encode(include_bytes!("../test_data/test-proof"));
        let inputs = WithdrawInputs {
            nullifier_hash: from_hex(
                "2478515297534FF5944224B019B82E8242B325B26624825D639C465B360AFFAF",
            )
            .unwrap()
            .try_into()
            .unwrap(),
            root: from_hex("5c7ad87d1e4f2a604b7d87e398e664a2fecc28c1d2a24f2226b0a9cd257d519a")
                .unwrap()
                .try_into()
                .unwrap(),
            proof: from_hex(&proof).unwrap().try_into().unwrap(),
            fee: 1u64,
            recipient: "5Gh8pDNFyir6ZdhkvNy2xGtfUNovRjxCzx5oMhhztXhGX3oZ".to_string(),
            relayer: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string(),
        };
        assert!(verify_proof(&inputs).await.is_ok());
    }

    #[tokio::test]
    #[should_panic]
    async fn test_proof_verification_error() {
        let proof = hex::encode(include_bytes!("../test_data/test-proof"));
        let inputs = WithdrawInputs {
            nullifier_hash: from_hex("2").unwrap().try_into().unwrap(),
            root: from_hex("5c7ad87d1e4f2a604b7d87e398e664a2fecc28c1d2a24f2226b0a9cd257d519a")
                .unwrap()
                .try_into()
                .unwrap(),
            proof: from_hex(&proof).unwrap().try_into().unwrap(),
            fee: 1u64,
            recipient: "5Gh8pDNFyir6ZdhkvNy2xGtfUNovRjxCzx5oMhhztXhGX3oZ".to_string(),
            relayer: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string(),
        };
        // verify_proof(&inputs).await.unwrap();
    }
}
