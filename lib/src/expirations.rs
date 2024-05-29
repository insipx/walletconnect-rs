//! Manages the event loop for expirations of keypairs

use const_format::concatcp;
use sled::Tree;

use crate::{WalletContext, STORAGE_PREFIX};

pub type Result<T> = std::result::Result<T, crate::error::ExpiryError>;
pub const EXPIRY: &str = "expiry";
pub const VERSION: u16 = 1;
pub const NAMESPACE: &str = concatcp!(STORAGE_PREFIX, ":", VERSION, "//", EXPIRY);

pub struct ExpiryManager {
    expirations: Tree,
}

impl ExpiryManager {
    pub fn new(context: &WalletContext) -> Result<Self> {
        let tree = context.db.open_tree(NAMESPACE)?;
        Ok(Self { expirations: tree })
    }

    pub fn set_expiry(&self, topic: &[u8; 32], expiry: u64) -> Result<()> {
        let bytes = rkyv::to_bytes::<_, 16>(&expiry)?;
        self.expirations.insert(topic, bytes.as_slice())?;
        Ok(())
    }

    pub fn get_expiry(&self, topic: &[u8; 32]) -> Result<Option<u64>> {
        let bytes = self.expirations.get(topic)?;

        Ok(match bytes {
            Some(bytes) => {
                let expiry = rkyv::from_bytes::<u64>(&bytes)?;
                Some(expiry)
            }
            None => None,
        })
    }
}
