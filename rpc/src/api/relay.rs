use crate::types::*;
use jsonrpsee::{
    // core::{client::Subscription, SubscriptionResult},
    proc_macros::rpc,
};

#[rpc(client)]
pub trait Relay {
    #[method(name = "publish")]
    fn publish(&self, topic: String, message: String, policy: Policy);

    #[subscription(name = "irn_subscribe", unsubscribe = "irn_unsubscribe", item = serde_json::Value)]
    fn relay_subscribe(&self, topic: String) -> SubscriptionResult;
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use crate::auth::AuthToken;
    use chrono::Duration;
    use crate::PROJECT_ID;

    #[tokio::test]
    async fn test_relay_subscribe() {
        
        let key = SigningKey::generate(&mut rand::thread_rng());
       
        let token = AuthToken::builder("http://example.com")
        .aud("wss://relay.walletconnect.com")
        .ttl(std::time::Duration::from_secs(60 * 60))
        .build()
        .as_jwt(&key)
        .unwrap();
        
        log::debug!("Token={token}");
        // let mut headers = HeaderMap::new();
        // headers.insert("Authorization", format!("Bearer {token}").parse().unwrap());
        // "wss://relay.walletconnect.com"
        let client = WsClientBuilder::default()
            // .set_headers(headers)
            .build(format!("wss://relay.walletconnect.com/?projectId={PROJECT_ID}&auth={token}"))
            .await;
        println!("Res: {:?}", client);
        client.unwrap();
    }
}
