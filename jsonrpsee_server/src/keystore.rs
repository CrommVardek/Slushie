use sc_keystore::LocalKeystore;
use sp_core::{crypto::Ss58Codec, offchain::KeyTypeId, sr25519::Public};
use sp_keystore::{CryptoStore, SyncCryptoStore};

/// Generate local key storage.
fn generate_store() -> Result<LocalKeystore, String> {
    let public_key =
        Public::from_ss58check("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
    let key_to: LocalKeystore =
        LocalKeystore::open("keystore", None).map_err(|err| err.to_string())?;
    let insert_key = |key_type, _public| {
        SyncCryptoStore::insert_unknown(&key_to, key_type, "//Alice", &public_key.0)
            .map_err(|_| format!("Failed to insert key"))
    };
    let seed = "//Alice";
    insert_key(KeyTypeId(*b"sr25"), seed)?;
    Ok(key_to)
}

/// Get public key.
pub async fn get_public_key(
    key_type_id: KeyTypeId,
    local_keystore: LocalKeystore,
) -> Result<Public, ()> {
    let local_keys = CryptoStore::sr25519_public_keys(&local_keystore, key_type_id).await;
    if !local_keys.is_empty() {
        Ok(local_keys[0].into())
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
