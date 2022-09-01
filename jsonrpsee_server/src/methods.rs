use crate::utils::verify_proof;
use anyhow::Error;
use async_once::AsyncOnce;
use jsonrpsee::types::error::CallError;
use lazy_static::lazy_static;
use shared::public_inputs::*;
use std::ops::Deref;
use subxt::{
    ext::{
        sp_core::{blake2_256, H256},
        sp_runtime::{app_crypto::Ss58Codec, scale_info::scale, AccountId32, MultiAddress},
    },
    tx::{Era, PairSigner, PlainTip, PolkadotExtrinsicParamsBuilder as Params},
    OnlineClient, PolkadotConfig,
};

lazy_static! {
    pub static ref API: AsyncOnce<OnlineClient<PolkadotConfig>> =
        AsyncOnce::new(async { OnlineClient::<PolkadotConfig>::new().await.unwrap() });
}
#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod node_runtime {}

/// Withdraw tokens.
pub async fn withdraw(
    signer: PairSigner<PolkadotConfig, sp_keyring::sr25519::sr25519::Pair>,
    inputs: WithdrawInputs,
) -> Result<H256, CallError> {
    let mut call_data = Vec::<u8>::new();
    call_data.append(&mut (&blake2_256("withdraw".as_bytes())[0..4]).to_vec());
    call_data.append(&mut scale::Encode::encode(&(
        &inputs.nullifier_hash,
        &inputs.root,
        &inputs.proof,
        inputs.fee,
        AccountId32::from_string(&inputs.recipient).map_err(|_| {
            CallError::InvalidParams(anyhow::Error::msg("Invalid recipient address."))
        })?,
    )));

    let tx = node_runtime::tx().contracts().call(
        MultiAddress::Id(
            AccountId32::from_string("5FrfL6HXGbETXUBegkx7vz7LWQ5MNWRGNYE7PEm1w176nLpz").map_err(
                |_| CallError::InvalidParams(anyhow::Error::msg("Invalid contract address.")),
            )?,
        ),
        0,
        49_000_000_000_000,
        None,
        call_data,
    );
    let tx_params = Params::new()
        .tip(PlainTip::new(0))
        .era(Era::Immortal, API.get().await.deref().genesis_hash());

    if verify_proof(&inputs).await.is_ok() {
        let tx_hash = API
            .get()
            .await
            .deref()
            .tx()
            .sign_and_submit(&tx, &signer, tx_params)
            .await
            .map_err(|_| CallError::Failed(anyhow::Error::msg("Transaction failed.")))?;
        return Ok(tx_hash);
    }

    return Err(CallError::Failed(anyhow::Error::msg(
        "Transaction canceled due to invalid proof.",
    )));
}

#[cfg(test)]

mod tests {
    use crate::methods::node_runtime;
    use crate::utils::verify_proof;
    use crate::withdraw;
    use dusk_bytes::Serializable;
    use futures::StreamExt;
    use shared::public_inputs::*;
    use shared::public_types::*;
    use shared::*;
    use sp_keyring::AccountKeyring;
    use subxt::events::Phase::ApplyExtrinsic;
    use subxt::ext::sp_core::bytes::from_hex;
    use subxt::tx::PairSigner;
    use subxt::{OnlineClient, PolkadotConfig};
    #[tokio::test]
    async fn test_withdraw() {
        let api = OnlineClient::<PolkadotConfig>::new().await.unwrap();
        let signer: PairSigner<PolkadotConfig, sp_keyring::sr25519::sr25519::Pair> =
            PairSigner::new(AccountKeyring::Alice.pair());
        let proof = hex::encode(include_bytes!("../test_data/test-proof"));
        let inputs = WithdrawInputs {
            nullifier_hash: from_hex(
                "12E4700B2A16A02D2E5CAF0DD78F09B5162D221A952799E838A3B01BA4AB228C",
            )
            .unwrap()
            .try_into()
            .unwrap(),
            root: from_hex("e0d769fc156408415cc18bf731e665a60eb2c380fd5c615a347af6350f652a1d")
                .unwrap()
                .try_into()
                .unwrap(),
            proof: from_hex(&proof).unwrap().try_into().unwrap(),
            fee: 1u64,
            recipient: "5Gh8pDNFyir6ZdhkvNy2xGtfUNovRjxCzx5oMhhztXhGX3oZ".to_string(),
            relayer: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string(),
        };

        if verify_proof(&inputs).await.is_ok() {
            withdraw(signer, inputs).await.unwrap();
        }

        let mut events = api
            .events()
            .subscribe()
            .await
            .unwrap()
            .filter_events::<(node_runtime::contracts::events::ContractEmitted,)>();

        let ev = events.next().await.unwrap();
        let details = ev.unwrap();
        assert_eq!(ApplyExtrinsic(1), details.phase);
    }
}
