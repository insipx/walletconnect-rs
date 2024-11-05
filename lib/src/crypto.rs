mod keychain;
pub use keychain::{Result as KeychainResult, *};
use sha2::Digest;

use crate::{types::Topic, WalletConnect};

pub type Result<T> = std::result::Result<T, crate::error::CryptoError>;

pub fn hash_sha256(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    hasher.finalize().into()
}

pub struct Crypto {
    keychain: Keychain,
}

impl Crypto {
    pub fn new(context: &WalletConnect) -> Result<Self> {
        Ok(Self { keychain: Keychain::new(context)? })
    }

    /// Set a symmetric key for a topic
    pub fn set_symkey(
        &self,
        key: [u8; 32],
        topic: Option<&Topic<'static>>,
    ) -> Result<Topic<'static>> {
        let topic = topic.unwrap_or_else(|| hex::encode(hash_sha256(&key)).into());
        let _ = self.keychain.set(topic, key)?;
        Ok(topic)
    }

    pub fn delete_symkey(&self, topic: Topic<'static>) -> Result<()> {
        self.keychain.delete(&topic)?;
        Ok(())
    }

    pub fn keychain(&self) -> &Keychain {
        &self.keychain
    }
}
