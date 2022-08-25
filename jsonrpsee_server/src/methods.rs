use async_once::AsyncOnce;
use lazy_static::lazy_static;
use shared::public_inputs::*;
use std::str::from_utf8;
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
    api: &OnlineClient<PolkadotConfig>,
    signer: PairSigner<PolkadotConfig, sp_keyring::sr25519::sr25519::Pair>,
    inputs: WithdrawInputs,
) -> Result<H256, Box<dyn std::error::Error>> {
    let mut call_data = Vec::<u8>::new();
    call_data.append(&mut (&blake2_256("withdraw".as_bytes())[0..4]).to_vec());
    call_data.append(&mut scale::Encode::encode(&(
        inputs.nullifier_hash,
        inputs.root,
        inputs.proof,
        inputs.fee,
        AccountId32::from_string(from_utf8(&inputs.recipient)?).unwrap(),
    )));

    let tx = node_runtime::tx().contracts().call(
        MultiAddress::Id(
            AccountId32::from_string("5DQZowXVkkcbjjZthsQ98q1uArBUFZ5sMqZyb1Xf4kknoHL6").unwrap(),
        ),
        0,
        900_000_000_000,
        None,
        call_data,
    );
    let tx_params = Params::new()
        .tip(PlainTip::new(0))
        .era(Era::Immortal, api.genesis_hash());

    let tx_hash = api.tx().sign_and_submit(&tx, &signer, tx_params).await?;
    Ok(tx_hash)
}
