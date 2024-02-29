use jsonrpsee::{proc_macros::rpc, types::ErrorObjectOwned, core::{client::Subscription, SubscriptionResult}, ws_client::WsClientBuilder};
use crate::types::prelude::*;


#[rpc(client)]
pub trait Relay {
    #[method(name = "publish")]
    fn publish(&self, topic: String, message: String, policy: Policy);

    #[subscription(name = "subscribe", item = serde_json::Value)]
    fn relay_subscribe(&self, topic: String);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_relay_subscribe() {
        // "wss://relay.walletconnect.com"
        let client = WsClientBuilder::default().build("wss://relay.walletconnect.com").await.unwrap();
    }

}
