pub mod keystore;
pub mod methods;
pub mod utils;

use jsonrpsee::{
    core::{server::access_control::AccessControlBuilder, Error},
    http_server::{HttpServerBuilder, HttpServerHandle, RpcModule},
    types::error::CallError,
};
use sp_keyring::AccountKeyring;
use std::net::SocketAddr;
use subxt::ext::sp_core::bytes::from_hex;
use subxt::{tx::PairSigner, PolkadotConfig};

use crate::methods::withdraw;
use shared::{public_inputs::*, public_types::*};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .expect("setting default subscriber failed");
    let (server_addr, _handle) = run_server().await?;
    println!("Run the following snippet in the developer console in any Website.");
    println!(
        r#"
        fetch("http://{}", {{
            method: 'POST',
            mode: 'cors',
            headers: {{ 'Content-Type': 'application/json' }},
            body: JSON.stringify({{
                jsonrpc: '2.0',
                method: 'withdraw',
                params: [nullifier_hash,
                         root,
                         proof,
                         fee,
                         recipient,
                         relayer 
                         ],
                id: 1 
            }})
        }}).then(res => {{
            console.log("Response:", res);
            return res.text()
        }}).then(body => {{
            console.log("Response Body:", body)
        }});
    "#,
        server_addr
    );
    futures::future::pending().await
}

/// Create RPC module with registered methods.
async fn setup_rpc_module() -> Result<RpcModule<()>, Error> {
    let mut module = RpcModule::new(());

    module.register_async_method("withdraw", |params, _| async move {
        let mut params_iter = params.parse::<Vec<String>>()?.into_iter();

        let nullifier_hash: PoseidonHash = from_hex(&params_iter.next().ok_or(
            CallError::InvalidParams(anyhow::Error::msg(
                "Nullifier Hash parameter is not provided.",
            )),
        )?)
        .map_err(|_| {
            CallError::InvalidParams(anyhow::Error::msg(
                "Cannot decode nullifier hash parameter.",
            ))
        })?
        .try_into()
        .map_err(|_| {
            CallError::InvalidParams(anyhow::Error::msg("Invalid nullifier hash parameter."))
        })?;

        let root: PoseidonHash = from_hex(&params_iter.next().ok_or(CallError::InvalidParams(
            anyhow::Error::msg("Root parameter is not provided."),
        ))?)
        .map_err(|_| CallError::InvalidParams(anyhow::Error::msg("Cannot decode root parameter.")))?
        .try_into()
        .map_err(|_| CallError::InvalidParams(anyhow::Error::msg("Invalid root parameter.")))?;

        let proof: SerializedProof = from_hex(&params_iter.next().ok_or(
            CallError::InvalidParams(anyhow::Error::msg("Proof parameter is not provided.")),
        )?)
        .map_err(|_| {
            CallError::InvalidParams(anyhow::Error::msg("Cannot decode proof parameter."))
        })?
        .try_into()
        .map_err(|_| CallError::InvalidParams(anyhow::Error::msg("Invalid proof parameter.")))?;

        let fee: u64 = params_iter
            .next()
            .ok_or(CallError::InvalidParams(anyhow::Error::msg(
                "Fee parameter is not provided.",
            )))?
            .parse::<u64>()
            .map_err(|_| CallError::InvalidParams(anyhow::Error::msg("Invalid fee parameter.")))?;

        let recipient: String =
            params_iter
                .next()
                .ok_or(CallError::InvalidParams(anyhow::Error::msg(
                    "Recipient parameter is not provided.",
                )))?;

        let relayer: String =
            params_iter
                .next()
                .ok_or(CallError::InvalidParams(anyhow::Error::msg(
                    "Relayer parameter is not provided.",
                )))?;

        let inputs = WithdrawInputs {
            nullifier_hash,
            root,
            proof,
            fee,
            recipient,
            relayer,
        };

        let signer: PairSigner<PolkadotConfig, sp_keyring::sr25519::sr25519::Pair> =
            PairSigner::new(AccountKeyring::Alice.pair());
        withdraw(signer, inputs)
            .await
            .map_err(|_| CallError::Failed(anyhow::Error::msg("RPC call failed. ")))?;

        Ok("OK".to_string())
    })?;

    Ok(module)
}

/// Run server.
async fn run_server() -> anyhow::Result<(SocketAddr, HttpServerHandle)> {
    let acl = AccessControlBuilder::new()
        .allow_all_headers()
        .allow_all_origins()
        .allow_all_hosts()
        .build();

    let server = HttpServerBuilder::default()
        .set_access_control(acl)
        .build("127.0.0.1:0".parse::<SocketAddr>()?)
        .await?;

    let addr = server.local_addr()?;
    let module = setup_rpc_module().await?;
    let server_handle = server.start(module)?;

    Ok((addr, server_handle))
}

#[cfg(test)]
mod tests {
    use crate::run_server;
    use jsonrpsee::core::client::ClientT;
    use jsonrpsee::http_client::HttpClientBuilder;
    use jsonrpsee::rpc_params;
    use jsonrpsee::types::error::CallError;
    use std::fs;
    use std::str::from_utf8;

    #[tokio::test]
    async fn test_client() {
        let (server_addr, _handle) = run_server().await.unwrap();
        let url = format!("http://{}", server_addr);
        let client = HttpClientBuilder::default().build(url).unwrap();
        let params = rpc_params!(
            "12E4700B2A16A02D2E5CAF0DD78F09B5162D221A952799E838A3B01BA4AB228C",
            "ac1d8ce6455937aea84f526141df0967c8bec03c7f47928c5be75fbbe18910b5",
            "81A146BD2DD8D5AFAE4DE592972226686321A63A589D268C1A5D90226339AB79DAEE6CA39DD61C4464A726637B96A39DB28DA37C949381AF3DEEFF5092E8A8E17CB63F639CEDFC803444CD1B6E29C0DBEE4800E3D362ABAB79563ECDF1BF0B08ACF3B31C770408D056691E1C0DD6217F9C35A66F3FDE31E542A4FC545A6384E1B6CECF977A77255E33995EDC409B06C6A1BEA66B71C2593D931856F59C256EE5C0E61DF1C60C16A7E713E0C4BB8B68EFE947DFD64783270B34274930768835488664290490349180003ECC5BB7EF014CA2C52ACB798BEACEF73B7C4CEFF0CAC0B9940E62DD76831D6BEC935655E3F7B685D0CCA9EF87691B06B316EDF4C3B15F8C2E90E447C0644CA224130EB4E1C3B53E3E7B495E590F646EC04F8496C9CA18AF951B79ABC937E25CA952F640604A0CACD6C4E6B74AA415FA36B062DF15B11A777D27EE5543958E5399B1619208D5188BCA955CF2158B67C782EA1EFBB303896162454E0BAFE2D35B9FBBF3FDC97104564A4EE142434D18CE0FB517F5DD73D0B2E6D9FBCDE70A9EC0A74B31A6A527EA7ABCB5BC7083849F7F32E83AABF534B5C55D2D7AE5AF6C0192D453BF706F807689E4DD67E82409A7881A93026920D7ED433FDE39C9EE12D981D3FCFA84E06E7BFECFA71302DB6AAC424E93A889B5AEBF8B899EB45008C67C68AF0F31BF25E48900D2BBD2B731994C92AEDAD463EF38A9AEC04A61B7CE571E2F2FCA9D4DFF1E98A8C011C9CEF920D5E7DBDCE66F1B0BBC7B403140785FE8669EEB2555A9773219D0FBE74E89341D5AFEE9FD08BB021C557EA0EE886396B03B0B4D700237416F2EEAF7722AA5B0642F50D7A50454334A7B8DDB028AEFAC2DA517A7A2665650844AAA7D131BB3563CEA31ACD6ACCE1F19D7087C45D0CA1B5E6FC760701D0C1637685CF22A54D9BBEFDC023D25EE62880E6E6A53F29C0BD785D24BB5B89707DBED1C7516BD65C5DF794BFF654FEB1FC5667E7F6FC48D25B1D573A473007AC57E363AEF89E0C2FA44BC7A429EB7F77A85406A400018C057441DA8F76E89B4B27E3D073CCCBF75AB2F81BF9EFC09F6B3B7120881A1301AF09AAA97C6EA26A863B9930E93113440180D088F3A58546E7EE49EC9769D34AB79C312C2290AC4F5C385431FCF0A249A381EFE5910FA4CCF24853FE4D450F5ADF1F7B81D29AAF2EC50FE734FA2FE46D566B38F085F40E0B21D253B81C35B7158BE42CBD464E2497EB2D3E43C968C47B5B9242B63F6AADFE09EE97FC4C155192EF0A689B38DAA1C659768A706AC423906986CE6D1160D49891B4A63C4361E1E50234252D13E800993DF7B2727C48A150222C2F4E76246804C642DA47F530C47FB2F80C738E1A110B531869F0A9C9D92DBCEE7830FD7DAB6E82AACFD27A77B9A1DC0D830E82CCC26CA27BC675686C6CACEA09830666AEF7F4A96D55FE99DBAB02E7AA56806669E1DED71379500",
            "1",
            "5Gh8pDNFyir6ZdhkvNy2xGtfUNovRjxCzx5oMhhztXhGX3oZ",
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        );

        let response: Result<String, _> = client.request("withdraw", params).await;
        assert_eq!("OK".to_string(), response.unwrap())
    }
}
