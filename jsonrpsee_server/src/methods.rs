use crate::utils::verify_proof;
use crate::WithdrawInputs;
use async_once::AsyncOnce;
use jsonrpsee::types::error::CallError;
use lazy_static::lazy_static;
use sp_keyring::sr25519::sr25519::Pair;
use std::ops::Deref;
use subxt::ext::sp_core::Pair as OtherPair;
use subxt::{
    ext::{
        sp_core::{blake2_256, H256},
        sp_runtime::{app_crypto::Ss58Codec, scale_info::scale, AccountId32, MultiAddress},
    },
    tx::{Era, PairSigner, PlainTip, PolkadotExtrinsicParamsBuilder as Params},
    OnlineClient, PolkadotConfig,
};
lazy_static! {
    pub static ref API: AsyncOnce<OnlineClient<PolkadotConfig>> = AsyncOnce::new(async {
        OnlineClient::<PolkadotConfig>::from_url("wss://rococo-contracts-rpc.polkadot.io:443")
            .await
            .unwrap()
    });
}
#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod node_runtime {}

pub async fn flip(seed: [u8; 32]) -> Result<H256, CallError> {
    let pair = Pair::from_seed(&seed);
    let signer: PairSigner<PolkadotConfig, Pair> = PairSigner::new(pair);
    let mut call_data = Vec::<u8>::new();
    call_data.append(&mut blake2_256("flip".as_bytes())[0..4].to_vec());

    let tx = node_runtime::tx().contracts().call(
        MultiAddress::Id(
            AccountId32::from_string("5Cy84KdQR7CdQhePxF68f669mwjgCX6t93VMHVmrJ4bbiwZM").map_err(
                |_| CallError::InvalidParams(anyhow::Error::msg("Invalid contract address.")),
            )?,
        ),
        0,
        20_000_000_000,
        None,
        call_data,
    );

    let tx_hash = API
        .get()
        .await
        .deref()
        .tx()
        .sign_and_submit_default(&tx, &signer)
        .await
        .unwrap();
    Ok(tx_hash)
}
/// Withdraw tokens.
pub async fn withdraw(
    signer: PairSigner<PolkadotConfig, sp_keyring::sr25519::sr25519::Pair>,
    inputs: WithdrawInputs,
) -> Result<H256, CallError> {
    let mut call_data = Vec::<u8>::new();
    call_data.append(&mut blake2_256("withdraw".as_bytes())[0..4].to_vec());
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

    Err(CallError::Failed(anyhow::Error::msg(
        "Transaction canceled due to invalid proof.",
    )))
}

#[cfg(test)]

mod tests {
    use crate::methods::{flip, node_runtime};
    use crate::public_inputs::WithdrawInputs;
    use crate::utils::verify_proof;
    use crate::withdraw;
    use futures::StreamExt;
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

    #[tokio::test]
    async fn test_flip() {
        let seed: [u8; 32] =
            from_hex("0xb945e93a978e6a5ffe7fa2b3f2ef807e8a8c972e2ee3801392adbba37ab6aa48")
                .unwrap()
                .try_into()
                .unwrap();
        let result = flip(seed).await;
        assert!(result.is_ok())
    }
}
