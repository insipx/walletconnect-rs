//! Pairing module for the WalletConnect pairing protocol.

mod types;
mod uri;

use chrono::Utc;
use const_format::concatcp;
use rand::{rngs::OsRng, RngCore};
use sled::Tree;
pub use uri::*;

use crate::{
    crypto::Crypto,
    rpc::{api::core::RelayClient, Client},
    time, WalletContext, STORAGE_PREFIX,
};

pub type Result<T> = std::result::Result<T, crate::error::PairingError>;
pub const PAIRING: &str = "pairing";
pub const VERSION: u16 = 1;
pub const NAMESPACE: &str = concatcp!(STORAGE_PREFIX, ":", VERSION, "//", PAIRING);

pub struct Pairing {
    pairings: Tree,
    crypto: Crypto,
    rpc: Client,
}

impl Pairing {
    pub fn new(context: &WalletContext) -> Result<Self> {
        let tree = context.db.open_tree(NAMESPACE)?;
        Ok(Self { pairings: tree, crypto: Crypto::new(context)?, rpc: context.rpc.clone() })
    }

    pub async fn create(&self) -> Result<()> {
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
            .expiry_timestamp(ttl)
            .build();
        self.pairings.insert(topic, uri.to_string().as_bytes())?;
        let subscription = self.rpc.inner().relay_subscribe(hex::encode(topic)).await?;

        // set expirer?
        Ok(())
    }
}
