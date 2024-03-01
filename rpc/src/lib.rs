pub mod api;
pub mod auth;
pub mod error;
pub mod types;

use jsonrpsee::{
    core::traits::IdKind,
    types::Id,
    ws_client::{WsClient, WsClientBuilder},
};
use rand::Rng as _;

use crate::{auth::AuthToken, error::ClientError};
use ed25519_dalek::{SigningKey, VerifyingKey};

// better-walletconnect-rs
pub const PROJECT_ID: &str = "c391bf7391b67ffbd8b8241389471ef8";

pub struct Client {
    client: WsClient<RequestIdGen>,
    key: SigningKey,
}

#[derive(Default, Copy, Clone)]
#[allow(dead_code)]
pub struct RequestIdGen;

impl IdKind for RequestIdGen {
    fn into_id(&self, _: u64) -> Id<'static> {
        let date = chrono::Utc::now().timestamp_millis() as u64;
        let entropy = 6;
        let date = date * 10_u64.pow(entropy);

        let extra = rand::thread_rng().gen_range(0..10_u64.pow(entropy));

        Id::Number(date + extra)
    }
}

impl Client {
    pub async fn new() -> Result<Self, ClientError> {
        let key = SigningKey::generate(&mut rand::thread_rng());

        let token = AuthToken::builder("http://example.com")
            .aud("wss://relay.walletconnect.com")
            .ttl(std::time::Duration::from_secs(60 * 60))
            .build()
            .as_jwt(&key)?;

        let client = WsClientBuilder::<RequestIdGen>::default()
            .id_format(RequestIdGen)
            .build(format!(
                "wss://relay.walletconnect.com/?projectId={PROJECT_ID}&auth={token}"
            ))
            .await?;

        Ok(Self { client, key })
    }

    /// Get the inner WsClient
    pub fn inner(&self) -> &WsClient<RequestIdGen> {
        &self.client
    }

    /// The the ed25519 public key associated with the session
    pub fn public_key(&self) -> VerifyingKey {
        self.key.verifying_key()
    }
}

#[cfg(test)]
mod test {
    use std::sync::Once;
    use tracing_subscriber::{
        fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry,
    };

    static INIT: Once = Once::new();

    #[ctor::ctor]
    fn __init_test_logging() {
        INIT.call_once(|| {
            let fmt = fmt::layer().compact();
            Registry::default()
                .with(EnvFilter::from_default_env())
                .with(fmt)
                .init()
        })
    }
}
