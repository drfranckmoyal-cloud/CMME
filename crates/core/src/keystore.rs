//! Clé de base : 32 octets aléatoires (générateur du système), rangés dans le trousseau macOS.
//! Jamais écrite dans un fichier en clair, jamais journalisée.

use crate::error::{CoreError, Result};
use zeroize::Zeroize;

pub trait KeyStore: Send + Sync {
    fn get(&self, account: &str) -> Result<Option<[u8; 32]>>;
    fn set(&self, account: &str, key: &[u8; 32]) -> Result<()>;
    fn delete(&self, account: &str) -> Result<()>;
}

pub fn generate_key() -> Result<[u8; 32]> {
    let mut k = [0u8; 32];
    getrandom::fill(&mut k).map_err(|e| CoreError::Keychain(format!("générateur aléatoire indisponible : {e}")))?;
    Ok(k)
}

fn decode(mut hex_str: String) -> Result<[u8; 32]> {
    let bytes = hex::decode(hex_str.trim()).map_err(|_| CoreError::Keychain("clé illisible dans le trousseau".into()));
    hex_str.zeroize();
    let bytes = bytes?;
    bytes.try_into().map_err(|_| CoreError::Keychain("clé de longueur inattendue".into()))
}

/// Trousseau macOS (Keychain), via la bibliothèque `keyring`.
pub struct KeychainStore {
    pub service: String,
}

impl KeyStore for KeychainStore {
    fn get(&self, account: &str) -> Result<Option<[u8; 32]>> {
        let entry = keyring::Entry::new(&self.service, account).map_err(|e| CoreError::Keychain(e.to_string()))?;
        match entry.get_password() {
            Ok(s) => decode(s).map(Some),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(CoreError::Keychain(e.to_string())),
        }
    }
    fn set(&self, account: &str, key: &[u8; 32]) -> Result<()> {
        let entry = keyring::Entry::new(&self.service, account).map_err(|e| CoreError::Keychain(e.to_string()))?;
        let mut h = hex::encode(key);
        let r = entry.set_password(&h).map_err(|e| CoreError::Keychain(e.to_string()));
        h.zeroize();
        r
    }
    fn delete(&self, account: &str) -> Result<()> {
        let entry = keyring::Entry::new(&self.service, account).map_err(|e| CoreError::Keychain(e.to_string()))?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(CoreError::Keychain(e.to_string())),
        }
    }
}

/// Coffre en mémoire, réservé aux tests automatiques (jamais utilisé par l'application).
#[derive(Default)]
pub struct MemoryKeyStore {
    inner: std::sync::Mutex<std::collections::HashMap<String, [u8; 32]>>,
}

impl KeyStore for MemoryKeyStore {
    fn get(&self, account: &str) -> Result<Option<[u8; 32]>> {
        Ok(self.inner.lock().unwrap().get(account).copied())
    }
    fn set(&self, account: &str, key: &[u8; 32]) -> Result<()> {
        self.inner.lock().unwrap().insert(account.to_string(), *key);
        Ok(())
    }
    fn delete(&self, account: &str) -> Result<()> {
        self.inner.lock().unwrap().remove(account);
        Ok(())
    }
}
