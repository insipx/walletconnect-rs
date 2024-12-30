//! Pairing modulecratei for the WalletConnect pairing protocol.

mod types;
mod uri;

use std::sync::Arc;

use chrono::Utc;
use const_format::concatcp;
use rand::{rngs::OsRng, RngCore};
use redb::TableDefinition;
use speedy::{Readable, Writable};
pub use uri::*;

pub use self::types::*;
use crate::{
    crypto::Crypto, error::PairingError, expirations::ExpiryManager, relayer::Relayer,
    rpc::prelude::RelayClient, time, types::Topic, WalletConnect, STORAGE_PREFIX,
};

pub type Result<T> = std::result::Result<T, PairingError>;

pub const PAIRING: &str = "pairing";
pub const VERSION: u16 = 1;
pub const NAMESPACE: &str = concatcp!(STORAGE_PREFIX, ":", VERSION, "//", PAIRING);
const TABLE: TableDefinition<&Topic, [u8; 32]> = TableDefinition::new(NAMESPACE);

fn to_fixed_bytes32<V: AsRef<[u8]>>(v: V) -> [u8; 32] {
    let mut fixed = [0u8; 32];
    let slice = v.as_ref();
    fixed.copy_from_slice(&slice[0..32]);
    fixed
}

pub struct Pairing {
    db: Arc<redb::Database>,
    crypto: Crypto,
    relayer: Relayer,
    expirer: ExpiryManager,
    events: crate::events::GlobalEvents,
    rpc: Arc<crate::rpc::Client>,
}

impl Pairing {
    pub fn new(context: &WalletConnect, events: crate::events::GlobalEvents) -> Result<Self> {
        let db = context.db.clone();
        let expirer = ExpiryManager::new(context)?;
        let relayer = Relayer::new(context);
        Ok(Self {
            db,
            relayer,
            crypto: Crypto::new(context)?,
            expirer,
            events,
            rpc: context.rpc.clone(),
        })
    }

    pub async fn create(&mut self) -> Result<(Topic<'static>, PairingUri)> {
        let sym_key: [u8; 32] = {
            let mut bytes = [0u8; 32];
            let mut rng = OsRng;
            rng.fill_bytes(&mut bytes);
            bytes
        };
        let topic = self.crypto.set_symkey(sym_key, None)?;
        // set a 5-minute TTL
        let ttl = Utc::now() + (time::MINUTE * 5);

        let uri = PairingUri::builder(topic.clone())
            .version(2)
            .protocol("irn")
            .symmetric_key(sym_key)
            .expiry_timestamp(ttl)
            .build();

        self.expirer.set_expiry(&topic, ttl.timestamp_millis())?;
        self.set(&topic, PairingMetadata::new(uri.clone(), false, None, Default::default()))?;
        self.relayer.subscribe(&topic).await?;

        Ok((topic, uri))
    }
    //  public pair: IPairing["pair"] = async (params) => {
    //    this.isInitialized();
    //    this.isValidPair(params);
    //    const { topic, symKey, relay, expiryTimestamp, methods } = parseUri(params.uri);
    //    let existingPairing;
    //    if (this.pairings.keys.includes(topic))
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
    //   // avoid overwriting keychain pairing already exists
    //    if (!this.core.crypto.keychain.has(topic)) {
    //      await this.core.crypto.setSymKey(symKey, topic);
    //    }
    //    await this.core.relayer.subscribe(topic, { relay });
    //    return pairing;
    //  };
    //

    /// Persist a pairing to the database
    pub fn set(&self, topic: &Topic<'static>, pairing: PairingMetadata) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        let bytes = pairing.write_to_vec()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            let _ret = table.insert(topic, to_fixed_bytes32(bytes))?;
            // ret.map(|v| v.value())
        };
        write_txn.commit()?;
        Ok(())
    }

    /// Get a persisted pairing
    pub fn get(&self, topic: &Topic<'static>) -> Result<Option<PairingMetadata>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        let bytes = table.get(topic)?.map(|v| v.value());

        Ok(match bytes {
            Some(bytes) => Some(PairingMetadata::read_from_buffer(&bytes)?.into_owned()),
            None => None,
        })
    }

    pub async fn pair(
        &self,
        uri: PairingUri<'static>,
        is_active: bool,
        activate: bool,
    ) -> Result<()> {
        use crate::error::Missing::*;

        // is initialized
        // is valid pair

        let (topic, sym_key, timestamp, _relay, _) = uri.decompose();
        if let Ok(Some(pairing)) = self.get(&topic) {
            if pairing.is_active() {
                // TODO: Error
                panic!("Pairing already exists");
            }
        }

        let default = crate::default_timestamp();
        let expiry = timestamp.unwrap_or(default);
        self.expirer.set_expiry(&topic, expiry)?;
        let pairing = PairingMetadata::new(uri.clone(), false, None, Default::default());
        self.set(&topic, pairing)?;

        if activate {
            self.activate(&topic).await?;
        }
        self.events.emit(PairingEvent::Create);

        if !self.crypto.keychain().contains(&topic)? {
            self.crypto
                .set_symkey(*sym_key.ok_or(PairingError::Missing(SymKeyParam))?, Some(&topic))?;
        }

        let subscription = self.rpc.client.relay_subscribe(topic.to_string());
        Ok(())
    }

    pub async fn activate(&self, topic: &Topic<'static>) -> Result<()> {
        // isInitialized? not sure if needed
        let expiry = Utc::now() + (crate::time::DAY * 30);
        self.expirer.set_expiry(topic, expiry.timestamp_millis())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[tokio::test]
    async fn test_subscribe() {
        let pairing_uri = PairingUri::from_str("wc:60f9a6f502ea7e82a4e9ad87e3f2e19b404a4905626f4df597f0cea588ee8a69@2?expiryTimestamp=1727121081&relay-protocol=irn&symKey=1c509d8c0c62dbe9c1ca93e2f6a021a13daf040562f438ee937bd483a8c9f983").unwrap();
        let wc = crate::WalletConnect::new("./test-db").await.unwrap();
        let mut subscription =
            wc.rpc.client.relay_subscribe(pairing_uri.topic.to_string()).await.unwrap();
    }
}
