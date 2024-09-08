use rkyv::{Archive, Deserialize, Serialize};

use crate::{pairing::uri::PairingUri, types::Metadata};

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
