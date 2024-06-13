//! Parser for Pairing URI Described [here](https://specs.walletconnect.com/2.0/specs/clients/core/pairing/pairing-uri)
//! EIP-1328 Compliant
//! uri         = "wc" ":" topic [ "@" version ][ "?" parameters ]
//! topic       = STRING
//! version     = 1*DIGIT
//! parameters  = parameter *( "&" parameter )
//! parameter   = key "=" value
//! key         = STRING
//! value       = STRING

use std::{collections::HashMap, fmt, str::FromStr};

use chrono::{DateTime, Utc};
use rkyv::Archive;
use serde::{Deserialize, Serialize};

use crate::{error::TypeError, Topic};

fn timestamp_to_datetime(secs: &str) -> DateTime<Utc> {
    DateTime::from_timestamp(secs.parse().expect("Must be digits"), 0).expect("Must be in range")
}

peg::parser! {
    grammar pairing_uri_parser() for str {
        pub rule uri() -> PairingUri
            = "wc:" t:topic() "@" v:version() "?" p:parameters() { PairingUri { topic: t, version: v.parse().expect("Must be u16"), parameters: Parameters(p) } }

        rule topic() -> String
            = t:$(unreserved()+) { t.to_string() }

        rule version() -> String
            = v:$(DIGIT()+) { v.to_string() }

        rule parameters() -> HashMap<Parameter, PairingParameter>
            = p:parameter() ++ "&" { p.into_iter().collect() }

        rule parameter() -> (Parameter, PairingParameter)
            = "expiryTimestamp=" v:$(DIGIT()+) { (Parameter::ExpiryTimestamp, PairingParameter::ExpiryTimestamp(timestamp_to_datetime(v))) }
            / "relay-protocol=" v:$(unreserved()+) { (Parameter::RelayProtocol, PairingParameter::RelayProtocol(v.to_string())) }
            / "symKey=" v:$(HEXDIG()*<64>) {
                let mut slice = [0u8; 32];
                hex::decode_to_slice(v, &mut slice).expect("Must be valid hex");
                (Parameter::SymKey, PairingParameter::SymKey(slice))
            }
            / k:$(unreserved()+) "=" v:$(unreserved()+) { (Parameter::Other(k.to_string()), PairingParameter::Other(v.to_string())) }

        rule unreserved() = ALPHA() / DIGIT() / "-" / "_" / "." / "~"

        rule ALPHA() -> char
            = ['a'..='z' | 'A'..='Z']

        rule DIGIT() -> char
            = ['0'..='9']

        rule HEXDIG() -> char
            = ['0'..='9' | 'a'..='f' | 'A'..='F']
    }
}

#[derive(
    Serialize,
    Deserialize,
    rkyv::Serialize,
    rkyv::Deserialize,
    rkyv::Archive,
    Debug,
    Clone,
    PartialEq,
)]
#[archive(check_bytes)]
pub struct PairingUri {
    pub topic: String,
    version: u16,
    #[serde(flatten)]
    parameters: Parameters,
}

// wc:3eeb85bb4f5e2c76ca9834be52d9444a418c71816ea3447cdad5ea1389010ac1@2?expiryTimestamp=1709402854&
// relay-protocol=irn&symKey=970246c97a4a60a5102d1807ff31dac0e2abd7165fd8d41a2b5b9093c7a46cee
impl PairingUri {
    pub fn builder(topic: Topic) -> PairingUriBuilder {
        PairingUriBuilder::new(topic)
    }

    pub fn parse<S: AsRef<str>>(uri: S) -> Result<PairingUri, TypeError> {
        pairing_uri_parser::uri(uri.as_ref()).map_err(Into::into)
    }

    pub fn param(&self, key: &Parameter) -> Option<&PairingParameter> {
        self.parameters.0.get(key)
    }

    //TODO: we do not handle `Other` correctly
    /// Decompose the PairingUri into its parts.
    /// The parts are in order of `Topic`, `SymKey`, `ExpiryTimestamp`, `RelayProtocol` and a
    /// [`Vec`] of any `String` representing [`PairingParameter::Other`]
    pub fn decompose(
        &self,
    ) -> (&String, Option<&[u8; 32]>, Option<&DateTime<Utc>>, Option<&String>, Vec<&String>) {
        (
            &self.topic,
            self.param(&Parameter::SymKey).map(|k| k.sym_key_unchecked()),
            self.param(&Parameter::ExpiryTimestamp).map(|t| t.expiry_timestamp_unchecked()),
            self.param(&Parameter::RelayProtocol).map(|r| r.relay_protocol_unchecked()),
            vec![],
        )
    }
}

pub struct PairingUriBuilder {
    pub topic: String,
    version: u16,
    parameters: HashMap<Parameter, PairingParameter>,
}

impl PairingUriBuilder {
    pub fn new(topic: Topic) -> Self {
        Self { topic: hex::encode(topic), version: 0, parameters: HashMap::new() }
    }

    pub fn version(mut self, version: u16) -> Self {
        self.version = version;
        self
    }

    pub fn protocol<S: AsRef<str>>(mut self, protocol: S) -> Self {
        let protocol: String = protocol.as_ref().into();
        self.parameters.insert(Parameter::RelayProtocol, PairingParameter::RelayProtocol(protocol));
        self
    }

    pub fn expiry_timestamp(mut self, time: DateTime<Utc>) -> Self {
        self.parameters.insert(Parameter::ExpiryTimestamp, PairingParameter::ExpiryTimestamp(time));
        self
    }

    pub fn symmetric_key(mut self, key: [u8; 32]) -> Self {
        self.parameters.insert(Parameter::SymKey, PairingParameter::SymKey(key));
        self
    }

    pub fn parameter(mut self, key: String, value: String) -> Self {
        self.parameters.insert(Parameter::Other(key), PairingParameter::Other(value));
        self
    }

    pub fn build(self) -> PairingUri {
        PairingUri {
            topic: self.topic,
            version: self.version,
            parameters: Parameters(self.parameters),
        }
    }
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    rkyv::Serialize,
    rkyv::Deserialize,
    rkyv::Archive,
    Clone,
    PartialEq,
    Eq,
)]
#[archive(check_bytes)]
pub struct Parameters(HashMap<Parameter, PairingParameter>);

impl fmt::Display for Parameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size = self.0.len();
        let mut visited = 0;

        for (k, v) in self.0.iter() {
            write!(f, "{k}={v}")?;
            visited += 1;
            if visited < size {
                write!(f, "&")?;
            }
        }
        Ok(())
    }
}

#[derive(
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    enum_as_inner::EnumAsInner,
)]
#[archive(check_bytes)]
pub enum PairingParameter {
    SymKey([u8; 32]),
    ExpiryTimestamp(DateTime<Utc>),
    RelayProtocol(String),
    Other(String),
}

impl fmt::Display for PairingParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SymKey(key) => write!(f, "{}", hex::encode(key)),
            Self::ExpiryTimestamp(date) => write!(f, "{}", date.timestamp()),
            Self::RelayProtocol(protocol) => write!(f, "{protocol}"),
            Self::Other(o) => write!(f, "{o}"),
        }
    }
}

impl PairingParameter {
    pub fn sym_key_unchecked(&self) -> &[u8; 32] {
        if let PairingParameter::SymKey(key) = self {
            key
        } else {
            panic!("unexpected; only decompose enum variant if certain of variant");
        }
    }

    pub fn expiry_timestamp_unchecked(&self) -> &DateTime<Utc> {
        if let PairingParameter::ExpiryTimestamp(timestamp) = self {
            timestamp
        } else {
            panic!("unexpected; only decompose enum variant if certain of variant");
        }
    }

    pub fn relay_protocol_unchecked(&self) -> &String {
        if let PairingParameter::RelayProtocol(protocol) = self {
            protocol
        } else {
            panic!("unexpected; only decompose enum variant if certain of variant");
        }
    }

    pub fn other_unchecked(&self) -> &String {
        if let PairingParameter::Other(o) = self {
            o
        } else {
            panic!("unexpected; only decompose enum variant if certain of variant");
        }
    }
}

#[derive(
    Serialize,
    Deserialize,
    rkyv::Serialize,
    rkyv::Deserialize,
    Archive,
    Clone,
    Hash,
    Debug,
    PartialEq,
    Eq,
    derive_more::Display,
)]
#[archive_attr(derive(Hash, PartialEq, Eq))]
#[archive(check_bytes)]
pub enum Parameter {
    #[display(fmt = "symKey")]
    SymKey,
    #[display(fmt = "expiryTimestamp")]
    ExpiryTimestamp,
    #[display(fmt = "relay-protocol")]
    RelayProtocol,
    #[display(fmt = "{}", _0)]
    Other(String),
}

impl<S: AsRef<str>> From<S> for Parameter {
    fn from(s: S) -> Self {
        match s.as_ref() {
            "symKey" => Self::SymKey,
            "expiryTimestamp" => Self::ExpiryTimestamp,
            "relay-protocol" => Self::RelayProtocol,
            o => Self::Other(o.to_string()),
        }
    }
}

impl fmt::Display for PairingUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { topic, version, parameters } = self;
        write!(f, "wc:{}@{}?{}", topic, version, parameters)
    }
}

impl FromStr for PairingUri {
    type Err = TypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PairingUri::parse(s)
    }
}

impl TryFrom<&str> for PairingUri {
    type Error = TypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        PairingUri::parse(value)
    }
}

impl TryFrom<String> for PairingUri {
    type Error = TypeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        PairingUri::parse(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pairing_uri() {
        let uri = "wc:topic@1?k1=v1&k2=v2&random=moreRandom";
        let _ = PairingUri::parse(uri).unwrap();
        let uri = "wc:1a41cdac130112b6f0cd364572bc71c2850256a0242ae9281616c0afefd09d15@2?expiryTimestamp=1709258103&relay-protocol=irn&symKey=6b71a77c2fa10e63886906e73e2f9471c285331cc6160e61b24e1abc8da71479";
        let result = pairing_uri_parser::uri(uri).unwrap();
        assert_eq!(
            result.topic,
            "1a41cdac130112b6f0cd364572bc71c2850256a0242ae9281616c0afefd09d15"
        );
        assert_eq!(result.version, 2);
        assert_eq!(result.parameters.0.len(), 3);
        let key = result.param(&Parameter::SymKey).and_then(PairingParameter::as_sym_key).unwrap();

        assert_eq!(
            hex::encode(key),
            "6b71a77c2fa10e63886906e73e2f9471c285331cc6160e61b24e1abc8da71479"
        );
    }

    #[test]
    fn test_pairing_uri_builder() {
        let now = Utc::now();
        let uri = PairingUri::builder([0u8; 32])
            .version(999)
            .protocol("irn")
            .symmetric_key([0u8; 32])
            .expiry_timestamp(now)
            .parameter("random".to_string(), "moreRandom".to_string())
            .build();

        let uri = uri.to_string();

        assert!(uri.contains(&format!("expiryTimestamp={}", now.timestamp())));
        assert!(uri.contains(&"relay-protocol=irn".to_string()));
        assert!(uri.contains(&"random=moreRandom".to_string()));
        assert!(uri.contains(&"symKey=00000000000000000000000000000000".to_string()));
        assert!(uri
            .contains("wc:0000000000000000000000000000000000000000000000000000000000000000@999?"));

        let uri = PairingUri::parse(uri).unwrap();
        assert_eq!(uri.param(&Parameter::SymKey), Some(&PairingParameter::SymKey([0u8; 32])));
        assert_eq!(
            uri.param(&Parameter::ExpiryTimestamp),
            Some(&PairingParameter::ExpiryTimestamp(
                DateTime::<Utc>::from_timestamp(now.timestamp(), 0).unwrap()
            ))
        );
        assert_eq!(
            uri.param(&Parameter::RelayProtocol),
            Some(&PairingParameter::RelayProtocol("irn".to_string()))
        );
    }
}
