use std::sync::Arc;

use crate::{
    error::RelayerError,
    rpc::{api::core::RelayClient, prelude::Subscription, Client},
    types::Topic,
    WalletConnect,
};

pub type Result<T> = std::result::Result<T, RelayerError>;

pub struct Relayer {
    subscriptions: Vec<Subscription<serde_json::Value>>,
    rpc: Client,
}

impl Relayer {
    pub fn new(context: &WalletConnect) -> Self {
        Self { rpc: Arc::unwrap_or_clone(context.rpc.clone()), subscriptions: vec![] }
    }

    pub async fn subscribe(&mut self, topic: &Topic<'static>) -> Result<()> {
        let subscription = self.rpc.inner().relay_subscribe(hex::encode(topic)).await?;
        self.subscriptions.push(subscription);
        Ok(())
    }
}
