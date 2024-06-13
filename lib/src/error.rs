use peg::str::LineCol;
use thiserror::Error;

/// Top-Level WalletConnect Error
#[derive(Error, Debug)]
pub enum WalletConnectError {
    #[error("DB Error {0}")]
    Db(#[from] sled::Error),
    #[error(transparent)]
    Rpc(#[from] walletconnect_rpc::error::ClientError),
}

/// TypeError which occurs during parsing of URIs
#[derive(Debug, Error)]
pub enum TypeError {
    #[error("Failed to parse pairing URI {0}")]
    Parse(#[from] peg::error::ParseError<LineCol>),
}

#[derive(Debug, Error)]
pub enum KeychainError {
    #[error("Error occured in Keychain Database Tree {0}")]
    Db(#[from] sled::Error),
}

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error(transparent)]
    Keychain(#[from] KeychainError),
}

#[derive(Debug, Error)]
pub enum PairingError {
    #[error(transparent)]
    Crypto(#[from] CryptoError),
    #[error("Error occured in Pairing Database Tree {0}")]
    Db(#[from] sled::Error),
    #[error(transparent)]
    Rpc(#[from] crate::rpc::error::JsonRpcError),
    #[error(transparent)]
    Expiry(#[from] ExpiryError),
    #[error(transparent)]
    Relayer(#[from] RelayerError),
    #[error("Rkyv Error {0}")]
    Rkyv(String),
}

#[derive(Debug, Error)]
pub enum ExpiryError {
    #[error("Error occured in Expiry Database Tree {0}")]
    Db(#[from] sled::Error),
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Error)]
pub enum RelayerError {
    #[error(transparent)]
    Rpc(#[from] crate::rpc::error::JsonRpcError),
}
