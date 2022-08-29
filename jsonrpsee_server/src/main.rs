pub mod keystore;
pub mod methods;

use jsonrpsee::{
    core::{server::access_control::AccessControlBuilder, Error},
    http_server::{HttpServerBuilder, HttpServerHandle, RpcModule},
    types::error::CallError,
};
use sp_keyring::AccountKeyring;
use std::net::SocketAddr;
use subxt::{ext::sp_core::bytes::from_hex, tx::PairSigner, PolkadotConfig};

use crate::methods::withdraw;
use shared::public_inputs::*;

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
                params: [nullifier_hash, root, fee, recipient],
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

        let nullifier_hash: PoseidonHash = params_iter
            .next()
            .ok_or(CallError::InvalidParams(anyhow::Error::msg(
                "Nullifier Hash parameter is not provided.",
            )))?
            .as_bytes()
            .try_into()
            .map_err(|_| {
                CallError::InvalidParams(anyhow::Error::msg("Invalid nullifier hash parameter."))
            })?;

        let root = from_hex(&params_iter.next().ok_or(CallError::InvalidParams(
            anyhow::Error::msg("Root parameter is not provided."),
        ))?)
        .map_err(|_| CallError::InvalidParams(anyhow::Error::msg("Invalid root parameter.")))?
        .try_into()
        .map_err(|_| CallError::InvalidParams(anyhow::Error::msg("Invalid root parameter.")))?;

        let proof = [0; 1040];

        let fee = params_iter
            .next()
            .ok_or(CallError::InvalidParams(anyhow::Error::msg(
                "Fee parameter is not provided.",
            )))?
            .parse::<u64>()
            .map_err(|_| CallError::InvalidParams(anyhow::Error::msg("Invalid fee parameter.")))?;

        let recipient = params_iter
            .next()
            .ok_or(CallError::InvalidParams(anyhow::Error::msg(
                "Recipient parameter is not provided.",
            )))?
            .as_bytes()
            .try_into()
            .map_err(|_| {
                CallError::InvalidParams(anyhow::Error::msg("Invalid recipient parameter."))
            })?;
        let inputs = WithdrawInputs {
            nullifier_hash,
            root,
            proof,
            fee,
            recipient,
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

    #[tokio::test]
    async fn test_client() {
        let (server_addr, _handle) = run_server().await.unwrap();
        let url = format!("http://{}", server_addr);
        let client = HttpClientBuilder::default().build(url).unwrap();
        let params = rpc_params!(
            "aaaaaaaaaaaapaaaaaaaaaazaaaaaaaa",
            "0x4ce946e968a0b477960eef24aafe0997350ba8f168ba2e4a546773556bdd1458",
            "1",
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        );
        let response: Result<String, _> = client.request("withdraw", params).await;

        assert_eq!("OK".to_string(), response.unwrap())
    }
}
