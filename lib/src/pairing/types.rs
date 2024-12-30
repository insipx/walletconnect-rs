use speedy::{Readable, Writable};

use crate::{pairing::uri::PairingUri, types::Metadata};

#[derive(Readable, Writable)]
pub struct PairingMetadata<'a> {
    uri: PairingUri<'a>,
    is_active: bool,
    peer_metadata: Option<Metadata>,
    methods: Vec<String>,
}

impl PairingMetadata<'_> {
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

    pub fn into_owned(self) -> PairingMetadata<'static> {
        PairingMetadata { uri: self.uri.into_owned(), ..self }
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
