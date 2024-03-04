use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Policy {
    pub ttl: u64,
    pub tag: u64,
}
