pub mod tr {
    use shared::public_inputs::*;
    use sp_keyring::AccountKeyring;
    use std::str::from_utf8;
    use subxt::ext::sp_runtime::scale_info::scale;
    use subxt::{
        ext::{
            sp_core::blake2_256,
            sp_runtime::{app_crypto::Ss58Codec, AccountId32, MultiAddress},
        },
        tx::{Era, PairSigner, PlainTip, PolkadotExtrinsicParamsBuilder as Params},
        OnlineClient, PolkadotConfig,
    };

    #[subxt::subxt(runtime_metadata_path = "metadata.scale")]
    pub mod polkadot {}

    pub async fn withdraw(inputs: WithdrawInputs) -> Result<(), Box<dyn std::error::Error>> {
        let api = OnlineClient::<PolkadotConfig>::new().await?;
        let signer: PairSigner<PolkadotConfig, sp_keyring::sr25519::sr25519::Pair> =
            PairSigner::new(AccountKeyring::Alice.pair());

        let mut call_data = Vec::<u8>::new();
        call_data.append(&mut (&blake2_256("withdraw".as_bytes())[0..4]).to_vec());
        call_data.append(&mut scale::Encode::encode(&(
            inputs.nullifier_hash,
            inputs.root,
            inputs.proof,
            inputs.fee,
            AccountId32::from_string(from_utf8(&inputs.recipient)?).unwrap(),
        )));

        let tx = polkadot::tx().contracts().call(
            MultiAddress::Id(
                AccountId32::from_string("5H1XJFnMAidtSApWY7N8bKVw3NxdFvDTimxhw4dxqt5DnU2X")
                    .unwrap(),
            ),
            0,
            900_000_000_000,
            None,
            call_data,
        );
        let tx_params = Params::new()
            .tip(PlainTip::new(0))
            .era(Era::Immortal, api.genesis_hash());

        // submit the transaction:
        let hash = api.tx().sign_and_submit(&tx, &signer, tx_params).await?;
        println!("Block hash {}", hash);
        Ok(())
    }
}

