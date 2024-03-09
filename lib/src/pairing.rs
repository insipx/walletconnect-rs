//! Pairing module for the WalletConnect pairing protocol.

mod types;
mod uri;

use std::collections::HashMap;

use chrono::Utc;
use rand::{rngs::OsRng, RngCore};
pub use uri::*;

use crate::{crypto::Crypto, time, StorageContext};

pub type Result<T> = std::result::Result<T, crate::error::PairingError>;

pub struct Pairing {
    pairings: HashMap<String, String>,
    crypto: Crypto,
}

impl Pairing {
    pub fn new(context: &StorageContext) -> Result<Self> {
        Ok(Self { pairings: HashMap::new(), crypto: Crypto::new(context)? })
    }

    pub fn create(&self) -> Result<()> {
        let sym_key: [u8; 32] = {
            let mut bytes = [0u8; 32];
            let mut rng = OsRng;
            rng.fill_bytes(&mut bytes);
            bytes
        };
        let topic = self.crypto.set_symkey(sym_key, None)?;
        // set a 5-minute TTL
        let ttl = Utc::now() + (time::MINUTE * 5);

        let uri = PairingUri::builder(topic)
            .version(2)
            .protocol("irn")
            .symmetric_key(sym_key)
            .expiry_timestamp(ttl);

        Ok(())
    }
}
