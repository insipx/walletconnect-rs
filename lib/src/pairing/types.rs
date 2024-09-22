use rkyv::{Archive, Deserialize, Serialize};

use crate::{expirations::ExpirationEvent, pairing::uri::PairingUri, types::Metadata};

#[derive(Archive, Deserialize, Serialize)]
#[archive(check_bytes)]
pub struct PairingMetadata {
    uri: PairingUri<'static>,
    is_active: bool,
    peer_metadata: Option<Metadata>,
    methods: Vec<String>,
}

impl PairingMetadata {
    pub fn new(
        uri: PairingUri<'static>,
        is_active: bool,
        peer_metadata: Option<Metadata>,
        methods: Vec<String>,
    ) -> Self {
        Self { uri, is_active, peer_metadata, methods }
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PairingEvent {
    Create,
    Expire,
    Delete,
    Ping,
}

impl From<PairingEvent> for crate::GlobalEvent {
    fn from(event: PairingEvent) -> crate::GlobalEvent {
        crate::GlobalEvent::Pairing(event)
    }
}
