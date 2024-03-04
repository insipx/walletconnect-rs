//! Parser for Pairing URI Described [here](https://specs.walletconnect.com/2.0/specs/clients/core/pairing/pairing-uri)
//! EIP-1328 Compliant
//! uri         = "wc" ":" topic [ "@" version ][ "?" parameters ]
//! topic       = STRING
//! version     = 1*DIGIT
//! parameters  = parameter *( "&" parameter )
//! parameter   = key "=" value
//! key         = STRING
//! value       = STRING

use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::error::TypeError;

peg::parser! {
    grammar pairing_uri_parser() for str {
        pub rule uri() -> PairingUri
            = "wc:" t:topic() "@" v:version() "?" p:parameters() { PairingUri { topic: t, version: v.parse().expect("Must be u16"), parameters: p } }

        rule topic() -> String
            = t:$(unreserved()+) { t.to_string() }

        rule version() -> String
            = v:$(DIGIT()+) { v.to_string() }

        rule parameters() -> HashMap<Parameter, PairingParameter>
            = p:parameter() ++ "&" { p.into_iter().collect() }

        rule parameter() -> (Parameter, PairingParameter)
            = "expiryTimestamp=" v:$(unreserved()+) { (Parameter::ExpiryTimestamp, PairingParameter::ExpiryTimestamp(v.parse().expect("Must be i64"))) }
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PairingUri {
    pub topic: String,
    version: u16,
    parameters: HashMap<Parameter, PairingParameter>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, enum_as_inner::EnumAsInner)]
pub enum PairingParameter {
    SymKey([u8; 32]),
    ExpiryTimestamp(i64),
    RelayProtocol(String),
    Other(String),
}

#[derive(Serialize, Deserialize, Clone, Hash, Debug, PartialEq, Eq)]
pub enum Parameter {
    SymKey,
    ExpiryTimestamp,
    RelayProtocol,
    Other(String),
}

// wc:3eeb85bb4f5e2c76ca9834be52d9444a418c71816ea3447cdad5ea1389010ac1@2?expiryTimestamp=1709402854&
// relay-protocol=irn&symKey=970246c97a4a60a5102d1807ff31dac0e2abd7165fd8d41a2b5b9093c7a46cee
impl PairingUri {
    pub fn parse<S: AsRef<str>>(uri: S) -> Result<PairingUri, TypeError> {
        pairing_uri_parser::uri(uri.as_ref()).map_err(Into::into)
    }

    pub fn get_param(&self, key: &Parameter) -> Option<&PairingParameter> {
        self.parameters.get(key)
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
        assert_eq!(result.parameters.len(), 3);
        let key =
            result.get_param(&Parameter::SymKey).and_then(PairingParameter::as_sym_key).unwrap();

        assert_eq!(
            hex::encode(key),
            "6b71a77c2fa10e63886906e73e2f9471c285331cc6160e61b24e1abc8da71479"
        );
    }
}
