pub mod keystore;
pub mod methods;
pub mod public_inputs;
pub mod utils;

use jsonrpsee::{
    core::{server::access_control::AccessControlBuilder, Error},
    http_server::{HttpServerBuilder, HttpServerHandle, RpcModule},
    types::error::CallError,
};
use public_inputs::WithdrawInputs;
use sp_keyring::AccountKeyring;
use std::net::SocketAddr;
use subxt::ext::sp_core::bytes::from_hex;
use subxt::{tx::PairSigner, PolkadotConfig};

use crate::methods::{flip, withdraw};
use shared::public_types::*;

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
                method: 'flip',
                params: [seed],
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

    module.register_async_method("flip", |params, _| async move {
        let mut params_iter = params.parse::<Vec<String>>()?.into_iter();
        let seed: [u8; 32] = from_hex(&params_iter.next().ok_or_else(|| {
            CallError::InvalidParams(anyhow::Error::msg("Seed parameter is not provided."))
        })?)
        .map_err(|_| CallError::InvalidParams(anyhow::Error::msg("Cannot decode seed parameter.")))?
        .try_into()
        .map_err(|_| CallError::InvalidParams(anyhow::Error::msg("Invalid seed parameter.")))?;
        flip(seed)
            .await
            .map_err(|_| CallError::Failed(anyhow::Error::msg("RPC call failed. ")))?;
        Ok("OK".to_string())
    })?;

    module.register_async_method("withdraw", |params, _| async move {
        let mut params_iter = params.parse::<Vec<String>>()?.into_iter();

        let nullifier_hash: PoseidonHash = from_hex(&params_iter.next().ok_or_else(|| {
            CallError::InvalidParams(anyhow::Error::msg(
                "Nullifier Hash parameter is not provided.",
            ))
        })?)
        .map_err(|_| {
            CallError::InvalidParams(anyhow::Error::msg(
                "Cannot decode nullifier hash parameter.",
            ))
        })?
        .try_into()
        .map_err(|_| {
            CallError::InvalidParams(anyhow::Error::msg("Invalid nullifier hash parameter."))
        })?;

        let root: PoseidonHash = from_hex(&params_iter.next().ok_or_else(|| {
            CallError::InvalidParams(anyhow::Error::msg("Root parameter is not provided."))
        })?)
        .map_err(|_| CallError::InvalidParams(anyhow::Error::msg("Cannot decode root parameter.")))?
        .try_into()
        .map_err(|_| CallError::InvalidParams(anyhow::Error::msg("Invalid root parameter.")))?;

        let proof: SerializedProof = from_hex(&params_iter.next().ok_or_else(|| {
            CallError::InvalidParams(anyhow::Error::msg("Proof parameter is not provided."))
        })?)
        .map_err(|_| {
            CallError::InvalidParams(anyhow::Error::msg("Cannot decode proof parameter."))
        })?
        .try_into()
        .map_err(|_| CallError::InvalidParams(anyhow::Error::msg("Invalid proof parameter.")))?;

        let fee: u64 = params_iter
            .next()
            .ok_or_else(|| {
                CallError::InvalidParams(anyhow::Error::msg("Fee parameter is not provided."))
            })?
            .parse::<u64>()
            .map_err(|_| CallError::InvalidParams(anyhow::Error::msg("Invalid fee parameter.")))?;

        let recipient: String = params_iter.next().ok_or_else(|| {
            CallError::InvalidParams(anyhow::Error::msg("Recipient parameter is not provided."))
        })?;

        let relayer: String = params_iter.next().ok_or_else(|| {
            CallError::InvalidParams(anyhow::Error::msg("Relayer parameter is not provided."))
        })?;

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

    #[tokio::test]
    async fn test_client() {
        let (server_addr, _handle) = run_server().await.unwrap();
        let url = format!("http://{}", server_addr);
        let client = HttpClientBuilder::default().build(url).unwrap();
        let params =
            rpc_params!("0xb945e93a978e6a5ffe7fa2b3f2ef807e8a8c972e2ee3801392adbba37ab6aa48");

        let response: Result<String, _> = client.request("flip", params).await;
        assert_eq!("OK".to_string(), response.unwrap())
    }
}
