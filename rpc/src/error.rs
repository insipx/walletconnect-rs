use peg::str::LineCol;
use thiserror::Error;
use jsonrpsee::core::ClientError as JsonRpcError;
use std::convert::Infallible;

#[derive(Debug, Error)]
pub enum TypeError {
    #[error("Failed to parse pairing URI {0}")]
    Parse(#[from] peg::error::ParseError<LineCol>),
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error(transparent)]
    Type(#[from] TypeError),
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    JsonRpc(#[from] JsonRpcError),
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error(transparent)]
    OutOrRange(#[from] chrono::OutOfRangeError),
}


impl From<Infallible> for ClientError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}