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
            AccountId32::from_string("5FT7RMg7FfsmGZg2MnrdSgdTbvu4r7ZCYfZLPh8v3iEL2MCP").map_err(
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
        let inputs = WithdrawInputs {
            nullifier_hash: from_hex("12E4700B2A16A02D2E5CAF0DD78F09B5162D221A952799E838A3B01BA4AB228C").unwrap().try_into().unwrap(),
            root: from_hex("e0d769fc156408415cc18bf731e665a60eb2c380fd5c615a347af6350f652a1d").unwrap().try_into().unwrap(),
            proof: from_hex("ABB115036DB24FFFC2667645E0E2E113255BA93609321B51A09837DC4AA5E006A8CAE037C35B21904C5A7F39E0AFD2D095AC398A56B26A42E94709C928DF59AEBE481BF683651FA369E6D6AB167E60D20426419661DE73A521E7C6DD5211346E85C375F173FFA8016A18A442F0B32E38AD11598587A0EB4F6AF0A618F9BDB64C5BB30EB6FEF93A9447113CCD1DE5D5AEB3E2D47BB73FF5D44FAC13AE52B4AE21C036245BCC69B25F0D343837E7891E6AE079B54B74BFF8A0228AC81D4D9A75A995DFEF2CD84E417568D3587BC835761B93EC21A500A9DA96255DD1831BE68AA8E7C0077DBEB0801D4E64DC4CD6596D908EF0557F300F5A41E5FADCEC53BC9828B7AB77E3DA2C5CB33BADE91CEFBE362FF9EC7B0B89F43530BBBEA0BA9A279FD7A6F3F1ED0881D620C2EBE7F967173A8A57390BB34C04FA38F4B5002C3F038E2275ECFD582B5DC98DE0A73378EAED15B18FC7EA72087F1B8B150C88FDB8F0BE1425580887AD7E4A885897EBB70A0F3D51775B16A707134FC095F899923A6128BEA8E0E4097C742AF390AA83D56FC695B9A213191E81D1AE8A8F26E2398F68AE3323C726D363215E7128C8FB9A581706C9A67F456EAABD670C951AAACE22708ADA4F28A44B4510B5CA114F0BC430E3B29BE3E8EFDBFB3E07E1D49E3C36D0FE79A48484E41EF43AE39216B52F846A2321ACC58466C7613FEDBB79C149A8724A28408F476C53D5E9C54DA0DCD1C01AFA6C35A7135AF66BA19D5DEF749FF0C8729994012C171A73C5540CAEB91E63E4573E6E333E6A4057AA9DB26E4601CDFF012D902ED17D010BD7552D4FEADA093BE2BA4FE3A2B607E0D68DE3B374306BB68C677D5CF188E9C56AC540CADEDB052C67443F3A52B79BE4D73D44D74B37E3DD2224313B4716E66F8FA83FBCFD9C08E6DCB524D0551AA7AE237B1CDF1B8FB597E7CD517A93C90B40E7F836F7C74EEE1197EE24772BBBB8ED52894CF3DB5814DC4AD51D13A7F032463D69B444B00A3D66773C2BEDC91F83EA0E675A01277893CFBE18ACCCED8CD630CA2423295EDEA4C5BFD15071103CDB2F5BEDD543FE40B0E8E0EB6F137140A4A3A8728BB72CFE90565700469C12205315829BF6DAFEA679DAF32DECC35346E49918B4B8E094547011A35867713FF6EBA954DF8800D00DF850EB1CACD54199D50C117739F467490723EDC61018C247600A1ED39BCC2760073FD14B0EF46955B936443C72D2BEA66C70EC5938536B7AA2C05CF6E86C418294D5F3B6E76E606AF69429F11036A23902A79FD262E06BF81121E8FF8FE944AAC0E8C34FECA0BDBCF78094EAA1225FA01CFAEFAD0AEB09AA2A3486F34FE2D20187BC8C41E8A150F2CC7CB0472D7C8469EBD328D2480045B0F08F438FA0CC806A9AD62B47150BA150F5431F58D557A20E7FE0E6EB6AF5D46ED04BD868804910AFF7C203F1949FC15F45886CF73B46DAFB348CB4BF4C")
                .unwrap()
                .try_into()
                .unwrap(),
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
