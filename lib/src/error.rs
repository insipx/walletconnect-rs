use peg::str::LineCol;
use thiserror::Error;

/// Top-Level WalletConnect Error
#[derive(Error, Debug)]
pub enum WalletConnectError {
    #[error("DB Error {0}")]
    Db(#[from] sled::Error),
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
}
