use std::collections::HashMap;

use jsonrpsee::proc_macros::rpc;

use crate::types::{common::*, sign::*};

// https://specs.walletconnect.com/2.0/specs/clients/sign/rpc-methods
#[rpc(client, namespace = "wc")]
pub trait Sign {
    // #[method(name = "sessionPropose", param_kind = map)]
    // fn session_propose(&self, propose: SessionPropose) -> RpcResult<SessionProposeResult>;

    #[method(name = "sessionPropose", param_kind = map)]
    fn session_propose(
        &self,
        relays: Vec<Relay>,
        proposer: Participant,
        requiredNamespaces: NamespaceMap,
    ) -> RpcResult<SessionProposeResult>;

    #[method(name = "sessionSettle")]
    fn session_settle(
        &self,
        relay: Relay,
        controller: Participant,
        namespaces: NamespaceMap,
        special_namespaces: Option<HashMap<String, Namespace>>,
        expiry: i64,
    ) -> RpcResult<bool>;

    #[method(name = "sessionUpdate")]
    fn session_update(&self, namespaces: NamespaceMap) -> RpcResult<bool>;

    #[method(name = "sessionExtend")]
    fn session_extend(&self, expiry: i64) -> RpcResult<bool>;

    #[method(name = "sessionRequest")]
    fn session_request(&self, request: Caip27Request, chain_id: String) -> RpcResult<bool>;

    #[method(name = "sessionEvent")]
    fn session_event(&self, event: Event) -> RpcResult<bool>;

    #[method(name = "sessionDelete")]
    fn session_delete(&self, code: i64, message: String) -> RpcResult<bool>;

    #[method(name = "sessionPing")]
    fn session_ping(&self) -> RpcResult<bool>;
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use jsonrpsee::core::{client::ClientT, params::ObjectParams};

    use super::*;
    use crate::test::with_client;

    #[tokio::test]
    async fn test_propose() -> Result<()> {
        with_client(|client| async move {
            let relays = vec![Relay::new("irn", None::<&str>)];
            let pubkey = client.public_key();

            let proposer = Participant::new(pubkey.to_bytes(), Metadata::default());

            // namespace for Sepolia and Mainnet
            // TODO: Create default namespaces
            let eip155 = Namespace {
                chains: vec!["eip155:1".into(), "eip155:11155111".into()],
                methods: vec![
                    "eth_sendTransaction".into(),
                    "eth_signTransaction".into(),
                    "eth_sign".into(),
                    "personal_sign".into(),
                    "eth_signTypedData".into(),
                ],
                events: vec!["chainChanged".into(), "acountsChanged".into()],
            };
            let mut required_namespaces = HashMap::new();
            required_namespaces.insert("eip155".to_string(), eip155);
            let result = client
                .inner()
                .session_propose(relays, proposer, required_namespaces)
                .await
                .unwrap();
            println!("result = {:?}", result);
            // assert_eq!(result, SessionProposeResult { status: "ok".to_string() });
            Ok(())
        })
        .await
    }
    
    /*
    #[tokio::test]
    async fn test_ping() -> Result<()> {
        with_client(|client| async move {

            log::debug!("Result={:?}", result);
            Ok(())
        })
        .await
    }
    */
}
