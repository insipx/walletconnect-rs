#![feature(trivial_bounds)]
use std::{path::Path, sync::Arc};

use chrono::Utc;
use rpc::Client;

use crate::error::WalletConnectError;

pub mod crypto;
pub mod error;
mod events;
mod expirations;
pub mod pairing;
mod relayer;
pub mod types;
pub use self::types::*;

/// RPC Api Re-Export
pub mod rpc {
    pub use walletconnect_rpc::*;
    // re-export jsonrpsee
}

pub const STORAGE_PREFIX: &str = "wc@2:core-rs";

/// The global Events loop
// static EVENTS: LazyLock<events::Events> = LazyLock::new(events::Events::new);

pub type Result<T> = std::result::Result<T, WalletConnectError>;

/// the global persistant storage type
#[derive(Clone, Debug)]
pub struct WalletConnect {
    db: Arc<redb::Database>,
    pub rpc: Arc<rpc::Client>,
}

impl WalletConnect {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let project_id = "684fc89c60a55ca93cd98576c86a73c9";
        let url = "https://github.com/insipx/walletconnect-rs-new";
        let rpc = Client::new(project_id, url).await?;
        Ok(Self { db: Arc::new(redb::Database::create(path)?), rpc })
    }
}

pub(crate) fn default_timestamp() -> i64 {
    (Utc::now() + (time::MINUTE * 5)).timestamp_millis()
}

/// The symmetric public key used for encryption
pub type SymKey = [u8; 32];

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
