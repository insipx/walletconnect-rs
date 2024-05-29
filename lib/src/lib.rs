use std::path::Path;

use rpc::Client;

use crate::error::WalletConnectError;

pub mod crypto;
pub mod error;
mod expirations;
pub mod pairing;

/// RPC Api Re-Export
pub mod rpc {
    pub use walletconnect_rpc::*;
    // re-export jsonrpsee
}

pub const STORAGE_PREFIX: &str = "wc@2:core-rs";

pub type Result<T> = std::result::Result<T, WalletConnectError>;

/// the global persistant storage type
#[derive(Clone, Debug)]
pub struct WalletContext {
    db: sled::Db,
    rpc: rpc::Client,
}

impl WalletContext {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let rpc = Client::new().await?;
        Ok(Self { db: sled::open(path)?, rpc })
    }
}

/// A Topic which is the Sha256 Hash of the Symmetric Key.
pub type Topic = [u8; 32];

/// Convenience Contants for time using second as the smallest unit.
#[allow(dead_code)]
mod time {
    use std::time::Duration;
    pub const NANOSECOND: Duration = Duration::from_nanos(1);
    pub const MICROSECOND: Duration = Duration::from_micros(1);
    pub const MILLISECOND: Duration = Duration::from_millis(1);
    pub const SECOND: Duration = Duration::from_secs(1);
    pub const MINUTE: Duration = SECOND.saturating_mul(60);
    pub const HOUR: Duration = MINUTE.saturating_mul(60);
    pub const DAY: Duration = HOUR.saturating_mul(24);
    pub const WEEK: Duration = DAY.saturating_mul(7);
    pub const MONTH: Duration = DAY.saturating_mul(30);
    pub const YEAR: Duration = DAY.saturating_mul(365);
}
