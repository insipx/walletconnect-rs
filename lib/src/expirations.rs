//! Manages the event loop for expirations of keypairs
use std::sync::Arc;

use const_format::concatcp;
use redb::TableDefinition;

use crate::{error::ExpiryError, types::Topic, WalletContext, STORAGE_PREFIX};

const TABLE: TableDefinition<&Topic, u64> = TableDefinition::new(NAMESPACE);

pub type Result<T> = std::result::Result<T, crate::error::ExpiryError>;
pub const EXPIRY: &str = "expiry";
pub const VERSION: u16 = 1;
pub const NAMESPACE: &str = concatcp!(STORAGE_PREFIX, ":", VERSION, "//", EXPIRY);

pub struct ExpiryManager {
    db: Arc<redb::Database>,
}

impl ExpiryManager {
    pub fn new(context: &WalletContext) -> Result<Self> {
        Ok(Self { db: context.db.clone() })
    }

    pub fn set_expiry(&self, topic: &Topic<'static>, expiry: u64) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            let _ret = table.insert(topic, expiry);
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_expiry(&self, topic: &Topic<'static>) -> Result<Option<u64>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        Ok(table.get(topic)?.map(|v| v.value()))
    }
}

#[derive(Clone, Copy)]
pub enum ExpirationEvent {
    TestEvent,
}
