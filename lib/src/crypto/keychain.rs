use const_format::concatcp;
use sled::Tree;

use crate::{Topic, WalletContext, STORAGE_PREFIX};

pub type Result<T> = std::result::Result<T, crate::error::KeychainError>;
pub const KEYCHAIN: &str = "keychain";
pub const VERSION: u16 = 1;
pub const NAMESPACE: &str = concatcp!(STORAGE_PREFIX, ":", VERSION, "//", KEYCHAIN);

/// Keychain storing a mapping of topics to symmetric diffie-hellman keys
pub struct Keychain {
    /// The namespaced Keychain database
    /// stores a mapping of Topics to symmetric encryption keys
    db: Tree,
}

fn to_fixed_bytes32<V: AsRef<[u8]>>(v: V) -> [u8; 32] {
    let mut fixed = [0u8; 32];
    let slice = v.as_ref();
    fixed.copy_from_slice(&slice[0..32]);
    fixed
}

impl Keychain {
    /// Instantiate a new Keychain
    pub fn new(context: &WalletContext) -> Result<Self> {
        let tree = context.db.open_tree(NAMESPACE)?;

        Ok(Self { db: tree })
    }

    /// Get the db-namespace this module uses
    pub const fn namespace() -> &'static str {
        NAMESPACE
    }

    /// Sets a topic to a symmetric key, and returns the old symmetric key if it existed.
    pub fn set(&self, topic: &Topic, key: [u8; 32]) -> Result<Option<[u8; 32]>> {
        Ok(self.db.insert(topic, &key)?.map(to_fixed_bytes32))
    }

    /// Get a symmetric key under a [`Topic`]
    pub fn get(&self, topic: &Topic) -> Result<Option<[u8; 32]>> {
        Ok(self.db.get(topic)?.map(to_fixed_bytes32))
    }

    /// Delete a [`Topic`] SymmetricKey mapping
    pub fn delete(&self, topic: &Topic) -> Result<()> {
        let _ = self.db.remove(topic)?;
        Ok(())
    }

    /// Check if the database contains a key
    pub fn contains(&self, topic: &Topic) -> Result<bool> {
        Ok(self.db.contains_key(topic)?)
    }
}
