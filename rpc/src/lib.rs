pub mod api;
pub mod auth;
pub mod error;
pub mod types;

use std::sync::Arc;

use ed25519_dalek::{SigningKey, VerifyingKey};
use jsonrpsee::{
    core::traits::IdKind,
    types::Id,
    ws_client::{WsClient, WsClientBuilder},
};
use rand::Rng as _;

use crate::{auth::AuthToken, error::ClientError};

// walletconnect-rs-new
pub const PROJECT_ID: &str = "c391bf7391b67ffbd8b8241389471ef8";

/// This is required to avoid request id collisions
pub const REQUEST_ID_ENTROPY: u32 = 6;

pub mod prelude {
    pub use jsonrpsee::core::client::Subscription;
    // pub use jsonrpsee::Subscription;
}

#[derive(Debug, Clone)]
pub struct Client {
    client: Arc<WsClient<RequestIdGen>>,
    key: SigningKey,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub struct RequestIdGen;

impl IdKind for RequestIdGen {
    fn into_id(&self, _: u64) -> Id<'static> {
        let date = chrono::Utc::now().timestamp_millis() as u64;
        let date = date * 10_u64.pow(REQUEST_ID_ENTROPY);

        let extra = rand::thread_rng().gen_range(0..10_u64.pow(REQUEST_ID_ENTROPY));

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
            .build(format!("wss://relay.walletconnect.com/?projectId={PROJECT_ID}&auth={token}"))
            .await?;

        let client = Arc::new(client);

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
    use std::{future::Future, sync::Once};

    use anyhow::Result;
    use tracing_subscriber::{
        fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry,
    };

    use super::*;

    static INIT: Once = Once::new();

    #[ctor::ctor]
    fn __init_test_logging() {
        INIT.call_once(|| {
            let fmt = fmt::layer().compact();
            Registry::default().with(EnvFilter::from_default_env()).with(fmt).init()
        })
    }

    pub async fn with_client<R>(fun: impl FnOnce(Client) -> R) -> Result<()>
    where
        R: Future<Output = Result<()>>,
    {
        let client = Client::new().await?;
        fun(client).await?;
        Ok(())
    }
}
