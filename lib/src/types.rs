//! Types
use std::{borrow::Cow, cmp::Ord, fmt};

use const_format::concatcp;
use speedy::{Readable, Writable};

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const TYPE_NAME: &str = concatcp!("{PKG_NAME}-TOPIC");

#[derive(Readable, Writable)]
pub struct Metadata {
    name: String,
    description: String,
    icons: Vec<String>,
    verification_url: Option<String>,
    /*
    redirect?: {
      native?: string;
      universal?: string;
    };
    */
}

#[derive(Clone, Copy)]
pub enum GlobalEvent {
    Pairing(super::pairing::PairingEvent),
    Expiration(super::expirations::ExpirationEvent),
}

/// A Topic, by default the sha256 hash of the symmetric key
/// but it _can_ be any string.
#[derive(
    Readable,
    Writable,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Topic<'a>(Cow<'a, str>);

impl<'a> Topic<'a> {
    pub fn new(s: &'a str) -> Self {
        Topic(Cow::Borrowed(s))
    }

    pub fn into_owned(self) -> Topic<'static> {
        Topic(self.0.into_owned().into())
    }
}

impl redb::Value for Topic<'static> {
    type SelfType<'a>
        = Topic<'static>
    where
        Self: 'a;

    type AsBytes<'a>
        = &'a str
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        <String as redb::Value>::fixed_width()
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Self(<String as redb::Value>::from_bytes(data).into())
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        use std::borrow::Cow::*;

        match &value.0 {
            Borrowed(b) => b,
            Owned(o) => o.as_str(),
        }
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new(TYPE_NAME)
    }
}

impl<'topic> redb::Value for &'topic Topic<'static> {
    type SelfType<'a>
        = Topic<'static>
    where
        Self: 'a;

    type AsBytes<'a>
        = &'a str
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        <String as redb::Value>::fixed_width()
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Topic(<String as redb::Value>::from_bytes(data).into())
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        use std::borrow::Cow::*;

        match &value.0 {
            Borrowed(b) => b,
            Owned(o) => o.as_str(),
        }
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new(TYPE_NAME)
    }
}

/*
impl redb::Value for &'static Topic<'_> {
    type SelfType<'a> = &'a Topic<'static>
    where
        Self: 'a;

    type AsBytes<'a> = &'a str
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        <String as redb::Value>::fixed_width()
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'static>
    where
        Self: 'a,
    {
        Topic(Cow::Borrowed(<&str as redb::Value>::from_bytes(data)))
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        <String as redb::Value>::as_bytes(&value.0)
    }

    fn type_name() -> redb::TypeName {
        <String as redb::Value>::type_name()
    }
}
*/

impl redb::Key for Topic<'static> {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        let topic1 = Topic::read_from_buffer(data1).unwrap();
        let topic2 = Topic::read_from_buffer(data2).unwrap();
        topic1.cmp(&topic2)
    }
}

impl redb::Key for &'static Topic<'static> {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        let topic1 = Topic::read_from_buffer(data1).unwrap();
        let topic2 = Topic::read_from_buffer(data2).unwrap();
        topic1.cmp(&topic2)
    }
}

impl From<String> for Topic<'static> {
    fn from(s: String) -> Topic<'static> {
        Topic(s.into())
    }
}

impl AsRef<[u8]> for Topic<'static> {
    fn as_ref(&self) -> &[u8] {
        &self.0.as_bytes()
    }
}

impl<'a> From<&'a str> for Topic<'a> {
    fn from(s: &'a str) -> Topic<'a> {
        Topic(Cow::Borrowed(s))
    }
}

impl fmt::Display for Topic<'static> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
