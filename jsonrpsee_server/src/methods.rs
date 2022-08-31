use async_once::AsyncOnce;
use jsonrpsee::types::error::CallError;
use lazy_static::lazy_static;
use plonk_prover::{verify, Pubkey};
use shared::{constants::DEFAULT_DEPTH, public_inputs::*, public_types::*};
use std::fs::File;
use std::io::Read;
use std::str::from_utf8;
use std::{fs, ops::Deref};
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
            AccountId32::from_string("5E43Yg8EnG52PPy9J2K5ERVt9bXirn3NFPG5r7YABeaKZSZ9").unwrap(),
        ),
        0,
        900_000_000_000,
        None,
        call_data,
    );
    let tx_params = Params::new()
        .tip(PlainTip::new(0))
        .era(Era::Immortal, API.get().await.deref().genesis_hash());

    let tx_hash = API
        .get()
        .await
        .deref()
        .tx()
        .sign_and_submit(&tx, &signer, tx_params)
        .await?;
    Ok(tx_hash)
}

/// Proof verification.
pub async fn proof_verification(
    //Nullifier hash
    h: String,
    //Root
    R: String,
    //Recipient address
    A: String,
    //Relayer address
    t: String,
    //Fee
    f: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let public_parameters = include_bytes!("test-correct-pp");
    let recipient: Pubkey = AccountId32::from_ss58check(&A)
        .map_err(|_| {
            CallError::InvalidParams(anyhow::Error::msg("Cannot decode recipient parameter."))
        })?
        .try_into()
        .map_err(|_| {
            CallError::InvalidParams(anyhow::Error::msg("Invalid recipient parameter."))
        })?;

    let relayer: Pubkey = AccountId32::from_ss58check(&t)
        .map_err(|_| {
            CallError::InvalidParams(anyhow::Error::msg("Cannot decode relayer parameter."))
        })?
        .try_into()
        .map_err(|_| CallError::InvalidParams(anyhow::Error::msg("Invalid relayer parameter.")))?;

    let root: [u8; 32] = hex::decode(R)
        .map_err(|_| CallError::InvalidParams(anyhow::Error::msg("Cannot decode root parameter.")))?
        .try_into()
        .map_err(|_| CallError::InvalidParams(anyhow::Error::msg("Invalid root parameter.")))?;

    let nullifier_hash: [u8; 32] = hex::decode(h)
        .map_err(|_| {
            CallError::InvalidParams(anyhow::Error::msg(
                "Cannot decode nullifier hash parameter.",
            ))
        })?
        .try_into()
        .map_err(|_| {
            CallError::InvalidParams(anyhow::Error::msg("Invalid nullifiier hash parameter."))
        })?;

    let proof: SerializedProof = fs::read("../plonk_prover_tool/test-proof")
        .map_err(|_| {
            CallError::InvalidParams(anyhow::Error::msg("Unable to read proof from a file."))
        })?
        .try_into()
        .map_err(|_| CallError::InvalidParams(anyhow::Error::msg("Invalid proof.")))?;

    verify::<DEFAULT_DEPTH>(
        public_parameters,
        nullifier_hash,
        root,
        recipient,
        relayer,
        f,
        &proof,
    )
    .map_err(|_| CallError::InvalidParams(anyhow::Error::msg("Invalid proof.")))?;
    Ok(())
}

#[cfg(test)]

mod tests {
    use super::proof_verification;
    use crate::methods::node_runtime;
    use crate::withdraw;
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
    async fn test_proof_verification_correct() {
        proof_verification(
            "29B3E77C5F6A9178BDCA67EEB33F8518F3C3277782698DA356F0384BA8C3E959".to_string(),
            "9f05ad0d662502e80ff0020a0184f4e3a74e1146fb7346694e49eb6cfeb01a8b".to_string(),
            "5Gh8pDNFyir6ZdhkvNy2xGtfUNovRjxCzx5oMhhztXhGX3oZ".to_string(),
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string(),
            1,
        )
        .await
        .unwrap();
    }
    #[tokio::test]
    #[should_panic]
    async fn test_proof_verification_error() {
        proof_verification(
            "1".to_string(),
            "8ba12c81e85a2c538113cb85c42886a8ecf46fe8f64daf875f8c1609d7306883".to_string(),
            "5ChyY5Rrn1ncJvUu77EpVDR6Ze74y1MT2ZSq6mjgMjbFgxda".to_string(),
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string(),
            1,
        )
        .await
        .unwrap();
    }
    #[tokio::test]
    async fn test_withdraw() {
        let api = OnlineClient::<PolkadotConfig>::new().await.unwrap();
        let signer: PairSigner<PolkadotConfig, sp_keyring::sr25519::sr25519::Pair> =
            PairSigner::new(AccountKeyring::Alice.pair());
        let inputs = WithdrawInputs {
            nullifier_hash: PoseidonHash::try_from("aaaaaaaaaaaiaaaaaaaaaaaaaaaaaaaz".as_bytes())
                .unwrap(),
            root: from_hex("0x4ce946e968a0b477960eef24aafe0997350ba8f168ba2e4a546773556bdd1458")
                .unwrap()
                .try_into()
                .unwrap(),
            proof: [0; 1040],
            fee: 1u64,
            recipient: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
                .as_bytes()
                .try_into()
                .unwrap(),
        };
        withdraw(signer, inputs).await.unwrap();

        let mut events = api
            .events()
            .subscribe()
            .await
            .unwrap()
            .filter_events::<(node_runtime::contracts::events::ContractEmitted,)>();

        let ev = events.next().await.unwrap();
        let details = ev.unwrap();
        //How can we get all hashes in specific block&
        //How can we check our extrinsic in block?
        assert_eq!(ApplyExtrinsic(1), details.phase);
    }
}
