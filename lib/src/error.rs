use std::fmt;

use peg::str::LineCol;
use thiserror::Error;

/// Top-Level WalletConnect Error
#[derive(Error, Debug)]
pub enum WalletConnectError {
    #[error(transparent)]
    Db(#[from] redb::DatabaseError),
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
    #[error(transparent)]
    Table(#[from] redb::TableError),
    #[error(transparent)]
    Db(#[from] redb::TransactionError),
    #[error(transparent)]
    Storage(#[from] redb::StorageError),
    #[error(transparent)]
    Commit(#[from] redb::CommitError),
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
    #[error(transparent)]
    Rpc(#[from] crate::rpc::error::JsonRpcError),
    #[error(transparent)]
    Expiry(#[from] ExpiryError),
    #[error(transparent)]
    Relayer(#[from] RelayerError),
    #[error("Rkyv Error {0}")]
    Rkyv(String),
    #[error(transparent)]
    Table(#[from] redb::TableError),
    #[error(transparent)]
    Db(#[from] redb::TransactionError),
    #[error(transparent)]
    Storage(#[from] redb::StorageError),
    #[error(transparent)]
    Commit(#[from] redb::CommitError),
    #[error(transparent)]
    Keychain(#[from] KeychainError),
    #[error("Missing {0}")]
    Missing(Missing),
    #[error(transparent)]
    Speedy(#[from] speedy::Error),
}

#[derive(Debug)]
pub enum Missing {
    SymKeyParam,
}

impl fmt::Display for Missing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Missing::SymKeyParam => write!(f, "symmetric key parameters"),
        }
    }
}

#[derive(Debug, Error)]
pub enum ExpiryError {
    #[error(transparent)]
    Table(#[from] redb::TableError),
    #[error(transparent)]
    Db(#[from] redb::TransactionError),
    #[error(transparent)]
    Storage(#[from] redb::StorageError),
    #[error(transparent)]
    Commit(#[from] redb::CommitError),
    #[error("{0}")]
    Other(String),
    #[error("conversion to millisecond timestamp failed")]
    MillisecondConversion,
}

#[derive(Debug, Error)]
pub enum RelayerError {
    #[error(transparent)]
    Rpc(#[from] crate::rpc::error::JsonRpcError),
}
