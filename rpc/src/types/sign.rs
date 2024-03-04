use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::common::*;

pub type NamespaceMap = HashMap<String, Namespace>;

/// The proposer/controller involved in this operation
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Participant {
    #[serde(
        serialize_with = "hex::serde::serialize",
        deserialize_with = "hex::serde::deserialize",
        rename = "publicKey"
    )]
    pub public_key: [u8; 32],
    pub metadata: Metadata,
}

impl Participant {
    pub fn new(public_key: impl Into<[u8; 32]>, metadata: Metadata) -> Self {
        Self { public_key: public_key.into(), metadata }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Namespace {
    pub chains: Vec<String>,
    pub methods: Vec<String>,
    pub events: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SessionProposeResult {
    relay: Relay,
    #[serde(rename = "responderPublicKey")]
    responder_public_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Caip27Request {
    method: String,
    params: serde_json::Value,
    expiry: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Event {
    name: String,
    data: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct SessionPropose {
    relays: Vec<Relay>,
    proposer: Participant,
    #[serde(rename = "requiredNamespaces")]
    required_namespaces: NamespaceMap,
}

impl From<(Vec<Relay>, Participant, NamespaceMap)> for SessionPropose {
    fn from(tuple: (Vec<Relay>, Participant, NamespaceMap)) -> Self {
        Self { relays: tuple.0, proposer: tuple.1, required_namespaces: tuple.2 }
    }
}
