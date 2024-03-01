use chrono::{DateTime, Utc};
use ed25519_dalek::{Signer, SigningKey};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, time::Duration};
use crate::error::AuthError;

pub const RELAY_WEBSOCKET_ADDRESS: &str = "wss://relay.walletconnect.com";

pub const MULTICODEC_ED25519_BASE: &str = "z";
pub const MULTICODEC_ED25519_HEADER: [u8; 2] = [237, 1];
pub const MULTICODEC_ED25519_LENGTH: usize = 32;

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtBasicClaims {
    pub iss: String,
    pub aud: String,
    pub sub: String,
    pub iat: i64,
    pub exp: Option<i64>,
}

pub const JWT_HEADER_TYP: &str = "JWT";
pub const JWT_HEADER_ALG: &str = "EdDSA";

#[derive(Serialize, Deserialize)]
pub struct JwtHeader<'a> {
    #[serde(borrow)]
    pub typ: &'a str,
    #[serde(borrow)]
    pub alg: &'a str,
}

impl Default for JwtHeader<'_> {
    fn default() -> Self {
        Self {
            typ: JWT_HEADER_TYP,
            alg: JWT_HEADER_ALG,
        }
    }
}

impl<'a> JwtHeader<'a> {
    pub fn is_valid(&self) -> bool {
        self.typ == JWT_HEADER_TYP && self.alg == JWT_HEADER_ALG
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SerializedAuthToken(String);

impl Display for SerializedAuthToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<SerializedAuthToken> for String {
    fn from(value: SerializedAuthToken) -> Self {
        value.0
    }
}

pub struct AuthToken {
    sub: String,
    aud: String,
    iat: DateTime<Utc>,
    ttl: Duration,
}

impl AuthToken {
    pub fn builder<S: AsRef<str>>(sub: S) -> AuthTokenBuilder {
        AuthTokenBuilder::new(sub.as_ref())
    }

    pub fn as_jwt(&self, key: &SigningKey) -> Result<SerializedAuthToken, AuthError> {
        let Self { sub, aud, iat, ttl } = self;
        encode_auth_token(key, sub, aud, iat, ttl)
    }
}

/// Builder for a JWT Auth Token according to [spec](https://specs.walletconnect.com/2.0/specs/clients/core/relay/relay-client-auth)
#[derive(Debug, Clone)]
pub struct AuthTokenBuilder {
    sub: String,
    aud: Option<String>,
    iat: Option<DateTime<Utc>>,
    ttl: Option<Duration>,
}

impl AuthTokenBuilder {
    pub fn new(sub: impl Into<String>) -> Self {
        Self {
            sub: sub.into(),
            aud: None,
            iat: None,
            ttl: None,
        }
    }

    pub fn aud(mut self, aud: impl Into<String>) -> Self {
        self.aud = Some(aud.into());
        self
    }

    pub fn iat(mut self, iat: impl Into<DateTime<Utc>>) -> Self {
        self.iat = Some(iat.into());
        self
    }

    pub fn ttl(mut self, ttl: impl Into<Duration>) -> Self {
        self.ttl = Some(ttl.into());
        self
    }

    pub fn build(self) -> AuthToken {
        let AuthTokenBuilder { sub, aud, iat, ttl } = self;

        AuthToken {
            sub: sub,
            aud: aud.unwrap_or(RELAY_WEBSOCKET_ADDRESS.to_string()),
            iat: iat.unwrap_or_else(Utc::now),
            ttl: ttl.unwrap_or_else(|| Duration::from_secs(3600)),
        }
    }
}

fn encode_auth_token(
    key: &SigningKey,
    sub: impl Into<String>,
    aud: impl Into<String>,
    iat: &DateTime<Utc>,
    ttl: &Duration,
) -> Result<SerializedAuthToken, AuthError> {
    let encoder = &data_encoding::BASE64URL_NOPAD;

    let exp = (*iat + chrono::Duration::from_std(*ttl)?).timestamp();

    let claims = {
        let data = JwtBasicClaims {
            iss: encode_key_as_did(key),
            sub: sub.into(),
            aud: aud.into(),
            iat: iat.timestamp(),
            exp: Some(exp),
        };
        log::debug!("Claims={}", serde_json::to_string_pretty(&data).unwrap());
        encoder.encode(serde_json::to_string(&data)?.as_bytes())
    };

    let header = encoder.encode(serde_json::to_string(&JwtHeader::default())?.as_bytes());
    let message = format!("{header}.{claims}");

    let signature = {
        let data = key.sign(message.as_bytes());

        encoder.encode(data.to_vec().as_slice())
    };

    Ok(SerializedAuthToken(format!("{message}.{signature}")))
}

pub fn encode_key_as_did(key: &SigningKey) -> String {
    let public_key = key.verifying_key();

    const PREFIX_LEN: usize = MULTICODEC_ED25519_HEADER.len();
    const TOTAL_LEN: usize = MULTICODEC_ED25519_LENGTH + PREFIX_LEN;

    let mut prefixed_data: [u8; TOTAL_LEN] = [0; TOTAL_LEN];
    prefixed_data[..PREFIX_LEN].copy_from_slice(&MULTICODEC_ED25519_HEADER);
    prefixed_data[PREFIX_LEN..].copy_from_slice(public_key.as_bytes());

    let encoded_data = bs58::encode(prefixed_data).into_string();

    let did = format!("did:key:{MULTICODEC_ED25519_BASE}{encoded_data}");
    log::debug!("did={did}");
    did
}
