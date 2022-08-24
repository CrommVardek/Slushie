use std::net::SocketAddr;
use std::ops::Deref;

use async_once::AsyncOnce;
use jsonrpsee::{
    core::server::access_control::AccessControlBuilder,
    http_server::{HttpServerBuilder, HttpServerHandle, RpcModule},
};
use lazy_static::lazy_static;
use random_string::generate;
use sp_keyring::AccountKeyring;
use subxt::ext::sp_core::bytes::from_hex;
use subxt::tx::PairSigner;
use subxt::{Error, OnlineClient, PolkadotConfig};

use jsonrpsee_server::withdraw;
use shared::public_inputs::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .expect("setting default subscriber failed");
    let charset = "abcdefghjklmnpoqrstuvyz";
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
                method: 'method_name',
                params: [{}, root, fee, recipient],
                id: 1
            }})
        }}).then(res => {{
            console.log("Response:", res);
            return res.text()
        }}).then(body => {{
            console.log("Response Body:", body)
        }});
    "#,
        server_addr,
        generate(32, charset)
    );
    futures::future::pending().await
}

lazy_static! {
    static ref API: AsyncOnce<OnlineClient<PolkadotConfig>> =
        AsyncOnce::new(async { OnlineClient::<PolkadotConfig>::new().await.unwrap() });
}

///Create RPC module with registered methods.
async fn setup_rpc_module() -> Result<RpcModule<()>, Error> {
    let mut module = RpcModule::new(());

    module.register_async_method("withdraw", |params, _| async move {
        let params_vec: Vec<String> = params.parse()?;
        let inputs = WithdrawInputs {
            nullifier_hash: params_vec[0]
                .as_bytes()
                .try_into()
                .expect("Invalid nullifierHash"),

            root: from_hex(&params_vec[1])
                .expect("")
                .try_into()
                .expect("Invalid root"),

            proof: [0; 1040],
            fee: params_vec[2].parse::<u64>().expect("Invalid fee"),
            recipient: params_vec[3].as_bytes().try_into().unwrap(),
        };

        let signer: PairSigner<PolkadotConfig, sp_keyring::sr25519::sr25519::Pair> =
            PairSigner::new(AccountKeyring::Alice.pair());
        withdraw(API.get().await.deref(), signer, inputs)
            .await
            .unwrap();

        Ok("good".to_string())
    })?;

    Ok(module)
}

///Run server.
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
