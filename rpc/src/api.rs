pub mod core;
mod sign;

use jsonrpsee::types::ErrorObjectOwned;
pub use sign::*;

pub type RpcResult<T> = Result<T, ErrorObjectOwned>;
