// use crate::types::pairing::*;

use jsonrpsee::proc_macros::rpc;

#[rpc(client, namespace = "wc")]
pub trait Pairing {
    #[method(name = "pairingDelete")]
    fn pairing_delete(&self, code: i64, message: String) -> Result<bool, ErrorObjectOwned>;

    #[method(name = "pairingPing")]
    fn pairing_ping(&self) -> Result<bool, ErrorObjectOwned>;

    #[method(name = "pairingExtend")]
    fn pairing_extend(&self, ttl: i64) -> Result<bool, ErrorObjectOwned>;
}

#[cfg(test)]
mod tests {}
