use jsonrpsee::proc_macros::rpc;

use crate::types::*;

#[rpc(client, namespace = "irn")]
pub trait Relay {
    #[method(name = "publish")]
    fn publish(&self, topic: String, message: String, policy: Policy);

    #[subscription(name = "subscribe", unsubscribe = "unsubscribe", item = serde_json::Value)]
    fn relay_subscribe(&self, topic: String) -> SubscriptionResult;
}

#[cfg(test)]
mod tests {
    use anyhow::Error;

    use crate::test::with_client;
    //use chrono::Duration;
    // use ed25519_dalek::SigningKey;

    #[tokio::test]
    async fn test_relay_subscribe() -> Result<(), Error> {
        with_client(|client| async move {
            println!("client={:?}", client);

            // let sub = client.relay_subscribe(topic).await.unwrap();
            Ok(())
        })
        .await?;
        Ok(())
    }
}
