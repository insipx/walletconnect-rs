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
use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};

use crate::{error::TypeError, types::Topic};

fn validate_timestamp(secs: &str) -> i64 {
    let stamp = DateTime::from_timestamp(secs.parse().expect("Must be digits"), 0)
        .expect("Must be in range");
    stamp.timestamp_millis()
}

peg::parser! {
    grammar pairing_uri_parser() for str {
        pub rule uri() -> PairingUri<'input>
            = "wc:" t:topic() "@" v:version() "?" p:parameters() { PairingUri { topic: Topic::new(t), version: v.parse().expect("Must be u16"), parameters: Parameters(p) } }

        rule topic() -> &'input str
            = t:$(unreserved()+) { t }

        rule version() -> String
            = v:$(DIGIT()+) { v.to_string() }

        rule parameters() -> HashMap<Parameter, PairingParameter>
            = p:parameter() ++ "&" { p.into_iter().collect() }

        rule parameter() -> (Parameter, PairingParameter)
            = "expiryTimestamp=" v:$(DIGIT()+) { (Parameter::ExpiryTimestamp, PairingParameter::ExpiryTimestamp(validate_timestamp(v))) }
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

#[derive(Serialize, Deserialize, Readable, Writable, Debug, Clone, PartialEq)]
pub struct PairingUri<'a> {
    pub topic: Topic<'a>,
    version: u16,
    #[serde(flatten)]
    parameters: Parameters,
}

// wc:3eeb85bb4f5e2c76ca9834be52d9444a418c71816ea3447cdad5ea1389010ac1@2?expiryTimestamp=1709402854&
// relay-protocol=irn&symKey=970246c97a4a60a5102d1807ff31dac0e2abd7165fd8d41a2b5b9093c7a46cee
impl<'a> PairingUri<'a> {
    pub fn builder(topic: Topic<'a>) -> PairingUriBuilder {
        PairingUriBuilder::new(topic)
    }

    /// Parse a pairing URI from an input `str`
    pub fn parse(uri: &'a str) -> Result<PairingUri<'a>, TypeError> {
        pairing_uri_parser::uri(uri).map_err(Into::into)
    }

    pub fn into_owned(self) -> PairingUri<'static> {
        let PairingUri { topic, version, parameters } = self;
        let topic = topic.into_owned();
        PairingUri { topic, version, parameters }
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
    ) -> (Topic<'static>, Option<&[u8; 32]>, Option<i64>, Option<&String>, Vec<&String>) {
        (
            self.topic.clone().into_owned(),
            self.param(&Parameter::SymKey).map(|k| k.sym_key_unchecked()),
            self.param(&Parameter::ExpiryTimestamp).map(|t| t.expiry_timestamp_unchecked()),
            self.param(&Parameter::RelayProtocol).map(|r| r.relay_protocol_unchecked()),
            vec![],
        )
    }
}

pub struct PairingUriBuilder<'a> {
    pub topic: Topic<'a>,
    version: u16,
    parameters: HashMap<Parameter, PairingParameter>,
}

impl<'a> PairingUriBuilder<'a> {
    pub fn new(topic: Topic<'a>) -> Self {
        Self { topic, version: 0, parameters: HashMap::new() }
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
        self.parameters.insert(
            Parameter::ExpiryTimestamp,
            PairingParameter::ExpiryTimestamp(time.timestamp_millis()),
        );
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

    pub fn build(self) -> PairingUri<'a> {
        PairingUri {
            topic: self.topic,
            version: self.version,
            parameters: Parameters(self.parameters),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Readable, Writable, Clone, PartialEq, Eq)]
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
    Clone,
    Serialize,
    Deserialize,
    Readable,
    Writable,
    Debug,
    PartialEq,
    Eq,
    enum_as_inner::EnumAsInner,
)]
pub enum PairingParameter {
    SymKey([u8; 32]),
    ExpiryTimestamp(i64),
    RelayProtocol(String),
    Other(String),
}

impl fmt::Display for PairingParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SymKey(key) => write!(f, "{}", hex::encode(key)),
            Self::ExpiryTimestamp(date) => write!(f, "{}", date),
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

    pub fn expiry_timestamp_unchecked(&self) -> i64 {
        if let PairingParameter::ExpiryTimestamp(timestamp) = self {
            *timestamp
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
    Clone,
    Readable,
    Writable,
    Serialize,
    Deserialize,
    Hash,
    Debug,
    PartialEq,
    Eq,
    derive_more::Display,
)]
pub enum Parameter {
    #[display("symKey")]
    SymKey,
    #[display("expiryTimestamp")]
    ExpiryTimestamp,
    #[display("relay-protocol")]
    RelayProtocol,
    #[display("{}", _0)]
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

impl fmt::Display for PairingUri<'static> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { topic, version, parameters } = self;
        write!(f, "wc:{}@{}?{}", topic, version, parameters)
    }
}

impl FromStr for PairingUri<'static> {
    type Err = TypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed = PairingUri::parse(s)?;
        Ok(parsed.into_owned())
    }
}

impl<'a> TryFrom<&'a str> for PairingUri<'a> {
    type Error = TypeError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        PairingUri::parse(value)
    }
}

impl TryFrom<String> for PairingUri<'static> {
    type Error = TypeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(PairingUri::parse(&value)?.into_owned())
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
            "1a41cdac130112b6f0cd364572bc71c2850256a0242ae9281616c0afefd09d15".into()
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
        let uri = PairingUri::builder(
            "0000000000000000000000000000000000000000000000000000000000000000".into(),
        )
        .version(999)
        .protocol("irn")
        .symmetric_key([0u8; 32])
        .expiry_timestamp(now.timestamp_millis())
        .parameter("random".to_string(), "moreRandom".to_string())
        .build();

        let uri = uri.to_string();
        println!("URI: {}", uri);
        assert!(uri.contains(&format!("expiryTimestamp={}", now.timestamp_millis())));
        assert!(uri.contains(&"relay-protocol=irn".to_string()));
        assert!(uri.contains(&"random=moreRandom".to_string()));
        assert!(uri.contains(&"symKey=00000000000000000000000000000000".to_string()));
        assert!(uri
            .contains("wc:0000000000000000000000000000000000000000000000000000000000000000@999?"));

        let uri = PairingUri::parse(&uri).unwrap();
        assert_eq!(uri.param(&Parameter::SymKey), Some(&PairingParameter::SymKey([0u8; 32])));
        assert_eq!(
            uri.param(&Parameter::ExpiryTimestamp),
            Some(&PairingParameter::ExpiryTimestamp(
                DateTime::<Utc>::from_timestamp(now.timestamp_millis(), 0).unwrap()
            ))
        );
        assert_eq!(
            uri.param(&Parameter::RelayProtocol),
            Some(&PairingParameter::RelayProtocol("irn".to_string()))
        );
    }
}
