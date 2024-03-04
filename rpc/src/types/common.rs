use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Relay {
    protocol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<String>,
}

impl Relay {
    pub fn new<S: Into<String>>(protocol: impl Into<String>, data: Option<S>) -> Self {
        Self { protocol: protocol.into(), data: data.map(Into::into) }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Metadata {
    name: String,
    description: String,
    url: Url,
    icons: Vec<Url>,
    #[serde(rename = "verifyUrl", skip_serializing_if = "Option::is_none")]
    verify_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect: Option<Redirect>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            name: "walletconnect-rs-new".into(),
            description: "walletconnect-rs-new SDK Comms".into(),
            url: Url::parse("https://github.com/insipx/walletconnect-rs-new")
                .expect("Static URL is correct"),
            icons: vec![
                Url::parse("https://www.rust-lang.org/static/images/rust-logo-blk.svg")
                    .expect("Static url is correct."),
                Url::parse("https://rustacean.net/assets/rustacean-flat-happy.svg")
                    .expect("Static url is correct."),
            ],
            verify_url: None,
            redirect: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Redirect {
    native: String,
    universal: String,
}
