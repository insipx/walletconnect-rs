//! Pairing modulecratei for the WalletConnect pairing protocol.

mod types;
mod uri;

use chrono::Utc;
use const_format::concatcp;
use rand::{rngs::OsRng, RngCore};
use sled::Tree;
pub use uri::*;

use self::types::*;
use crate::{
    crypto::Crypto, error::PairingError, expirations::ExpiryManager, relayer::Relayer, time,
    WalletContext, STORAGE_PREFIX,
};

pub type Result<T> = std::result::Result<T, PairingError>;
pub const PAIRING: &str = "pairing";
pub const VERSION: u16 = 1;
pub const NAMESPACE: &str = concatcp!(STORAGE_PREFIX, ":", VERSION, "//", PAIRING);

pub struct Pairing {
    pairings: Tree,
    crypto: Crypto,
    relayer: Relayer,
    expirer: ExpiryManager,
}

impl Pairing {
    pub fn new(context: &WalletContext) -> Result<Self> {
        let tree = context.db.open_tree(NAMESPACE)?;
        let expirer = ExpiryManager::new(context)?;
        let relayer = Relayer::new(context);

        Ok(Self { pairings: tree, relayer, crypto: Crypto::new(context)?, expirer })
    }

    pub async fn create(&mut self) -> Result<([u8; 32], PairingUri)> {
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

        self.expirer.set_expiry(&topic, ttl.timestamp_millis() as u64)?;
        self.set(&topic, PairingMetadata::new(uri.clone(), false, None, Default::default()))?;
        self.relayer.subscribe(&topic).await?;

        Ok((topic, uri))
    }
    //  public pair: IPairing["pair"] = async (params) => {
    //    this.isInitialized();
    //    this.isValidPair(params);
    //    const { topic, symKey, relay, expiryTimestamp, methods } = parseUri(params.uri);
    //    let existingPairing;
    //    if (this.pairings.keys.includes(topic)) {
    //      existingPairing = this.pairings.get(topic);
    //      if (existingPairing.active) {
    //        throw new Error(
    //          `Pairing already exists: ${topic}. Please try again with a new connection URI.`,
    //        );
    //      }
    //    }
    //
    //    const expiry = expiryTimestamp || calcExpiry(FIVE_MINUTES);
    //    const pairing = { topic, relay, expiry, active: false, methods };
    //    this.core.expirer.set(topic, expiry);
    //    await this.pairings.set(topic, pairing);
    //
    //    if (params.activatePairing) {
    //      await this.activate({ topic });
    //    }
    //
    //    this.events.emit(PAIRING_EVENTS.create, pairing);
    //
    //    // avoid overwriting keychain pairing already exists
    //    if (!this.core.crypto.keychain.has(topic)) {
    //      await this.core.crypto.setSymKey(symKey, topic);
    //    }
    //    await this.core.relayer.subscribe(topic, { relay });
    //    return pairing;
    //  };
    //

    pub fn set(&self, topic: &[u8; 32], pairing: PairingMetadata) -> Result<()> {
        let bytes =
            rkyv::to_bytes::<_, 16>(&pairing).map_err(|e| PairingError::Rkyv(e.to_string()))?;
        self.pairings.insert(topic, bytes.as_slice())?;
        Ok(())
    }

    pub fn get(&self, topic: &[u8; 32]) -> Result<Option<PairingMetadata>> {
        // let bytes = rkyv::to_bytes::<_, 16>(&
        let bytes = self.pairings.get(topic)?;
        Ok(match bytes {
            Some(bytes) => {
                let pairing = rkyv::from_bytes::<PairingMetadata>(&bytes)
                    .map_err(|e| PairingError::Rkyv(e.to_string()))?;
                Some(pairing)
            }
            None => None,
        })
    }

    pub async fn pair(&self, uri: PairingUri, is_active: bool) -> Result<()> {
        // is initialized
        // is valid pair
        let (topic, sym_key, timestamp, relay, _) = uri.decompose();
        /*
                if let Some(pairing) = self.get(topic) {
                    // if pairing.
                }
        */
        Ok(())
    }
}
