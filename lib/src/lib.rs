use std::path::Path;

use crate::error::WalletConnectError;

pub mod crypto;
pub mod error;
pub mod pairing;

pub const STORAGE_PREFIX: &str = "wc@2:core-rs";

pub type Result<T> = std::result::Result<T, WalletConnectError>;

/// the global persistant storage type
#[derive(Clone, Debug)]
pub struct StorageContext {
    db: sled::Db,
}

impl StorageContext {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self { db: sled::open(path)? })
    }
}

/// A Topic which is the Sha256 Hash of the Symmetric Key.
pub type Topic = [u8; 32];

/// Convenience Contants for time using second as the smallest unit.
mod time {
    use std::time::Duration;

    pub const SECOND: Duration = Duration::from_secs(1);
    pub const MINUTE: Duration = SECOND.saturating_mul(60);
    pub const HOUR: Duration = MINUTE.saturating_mul(60);
    pub const DAY: Duration = HOUR.saturating_mul(24);
}
