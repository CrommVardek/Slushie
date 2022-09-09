use sc_keystore::LocalKeystore;
use sp_core::{crypto::Ss58Codec, offchain::KeyTypeId, sr25519::Public};
use sp_keystore::{CryptoStore, SyncCryptoStore};

fn generate_store() -> Result<LocalKeystore, String> {
    let public_key =
        Public::from_ss58check("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
    let key_to: LocalKeystore =
        LocalKeystore::open("keystore", None).map_err(|err| err.to_string())?;
    let _ = SyncCryptoStore::insert_unknown(&key_to, KeyTypeId(*b"sr25"), "//Alice", &public_key.0)
        .map_err(|_| "Failed to add key".to_string());
    Ok(key_to)
}

pub async fn get_public_key(
    key_type_id: KeyTypeId,
    local_keystore: LocalKeystore,
) -> Result<Public, ()> {
    let local_keys = CryptoStore::sr25519_public_keys(&local_keystore, key_type_id).await;
    if !local_keys.is_empty() {
        Ok(local_keys[0])
    } else {
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn keystore_generated() {
        let _public_key = match generate_store() {
            Ok(keystore) => get_public_key(KeyTypeId(*b"sr25"), keystore).await,
            Err(string) => panic!("{}", string),
        };
    }
}
