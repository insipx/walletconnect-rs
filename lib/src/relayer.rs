use crate::{
    error::RelayerError,
    rpc::{api::core::RelayClient, prelude::Subscription, Client},
    WalletContext,
};

pub type Result<T> = std::result::Result<T, RelayerError>;

pub struct Relayer {
    subscriptions: Vec<Subscription<serde_json::Value>>,
    rpc: Client,
}

impl Relayer {
    pub fn new(context: &WalletContext) -> Self {
        Self { rpc: context.rpc.clone(), subscriptions: vec![] }
    }

    pub async fn subscribe(&mut self, topic: &[u8; 32]) -> Result<()> {
        let subscription = self.rpc.inner().relay_subscribe(hex::encode(topic)).await?;
        self.subscriptions.push(subscription);
        Ok(())
    }
}
