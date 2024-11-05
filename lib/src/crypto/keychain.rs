use std::sync::Arc;

use const_format::concatcp;
use redb::{Database, TableDefinition};

use crate::{types::Topic, WalletConnect, STORAGE_PREFIX};

const TABLE: TableDefinition<&Topic, [u8; 32]> = TableDefinition::new(NAMESPACE);

pub type Result<T> = std::result::Result<T, crate::error::KeychainError>;
pub const KEYCHAIN: &str = "keychain";
pub const VERSION: u16 = 1;
pub const NAMESPACE: &str = concatcp!(STORAGE_PREFIX, ":", VERSION, "//", KEYCHAIN);

/// Keychain storing a mapping of topics to symmetric diffie-hellman keys
pub struct Keychain {
    /// The namespaced Keychain database
    /// stores a mapping of Topics to symmetric encryption keys
    db: Arc<Database>,
}

// TODO: Make this into a trait for tables in redb
impl Keychain {
    /// Instantiate a new Keychain
    pub fn new(context: &WalletConnect) -> Result<Self> {
        Ok(Self { db: context.db.clone() })
    }

    /// Get the db-namespace this module uses
    pub const fn namespace() -> &'static str {
        NAMESPACE
    }

    /// Sets a topic to a symmetric key, and returns the old symmetric key if it existed.
    pub fn set(&self, topic: &Topic<'static>, key: [u8; 32]) -> Result<Option<[u8; 32]>> {
        let write_txn = self.db.begin_write()?;
        let old_value = {
            let mut table = write_txn.open_table(TABLE)?;
            let ret = table.insert(topic, key)?;
            ret.map(|v| v.value())
        };
        write_txn.commit()?;
        Ok(old_value)
    }

    /// Get a symmetric key under a [`Topic`]
    pub fn get(&self, topic: &Topic<'static>) -> Result<Option<[u8; 32]>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        Ok(table.get(topic)?.map(|v| v.value()))
    }

    /// Delete a [`Topic`] SymmetricKey mapping
    pub fn delete(&self, topic: &Topic<'static>) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            let _value_guard = table.remove(topic)?;
        }
        write_txn.commit()?;

        Ok(())
    }

    /// Check if the database contains a key
    pub fn contains(&self, topic: &Topic<'static>) -> Result<bool> {
        Ok(self.get(topic)?.is_some())
    }
}
