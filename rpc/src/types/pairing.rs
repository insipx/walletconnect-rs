//! Parser for Pairing URI Described [here](https://specs.walletconnect.com/2.0/specs/clients/core/pairing/pairing-uri)
//! EIP-1328 Compliant
//! uri         = "wc" ":" topic [ "@" version ][ "?" parameters ]
//! topic       = STRING
//! version     = 1*DIGIT
//! parameters  = parameter *( "&" parameter )
//! parameter   = key "=" value
//! key         = STRING
//! value       = STRING

use crate::error::TypeError;
use std::str::FromStr;

peg::parser! {
    grammar pairing_uri_parser() for str {
        pub rule uri() -> PairingUri
            = "wc:" t:topic() "@" v:version() "?" p:paramaters() { PairingUri { topic: t, version: v.parse().expect("Must be u16"), parameters: p } }

        rule topic() -> String
            = t:$(unreserved()+) { t.to_string() }

        rule version() -> String
            = v:$(DIGIT()+) { v.to_string() }

        rule paramaters() -> Vec<(String, String)>
            = p:parameter() ++ "&" { p }

        rule parameter() -> (String, String)
            = k:$(unreserved()+) "=" v:$(unreserved()+) { (k.to_string(), v.to_string()) }

        rule unreserved() = ALPHA() / DIGIT() / "-" / "_" / "." / "~"

        rule ALPHA() -> char
            = ['a'..='z' | 'A'..='Z']

        rule DIGIT() -> char
            = ['0'..='9']
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PairingUri {
    pub topic: String,
    version: u16,
    parameters: Vec<(String, String)>,
}

impl PairingUri {
    pub fn parse<S: AsRef<str>>(uri: S) -> Result<PairingUri, TypeError> {
        pairing_uri_parser::uri(uri.as_ref()).map_err(Into::into)
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
        let result = pairing_uri_parser::uri(uri).unwrap();
        let uri = "wc:1a41cdac130112b6f0cd364572bc71c2850256a0242ae9281616c0afefd09d15@2?expiryTimestamp=1709258103&relay-protocol=irn&symKey=6b71a77c2fa10e63886906e73e2f9471c285331cc6160e61b24e1abc8da71479";
        let result = pairing_uri_parser::uri(uri).unwrap();
        println!("{:#?}", result)
    }
}
