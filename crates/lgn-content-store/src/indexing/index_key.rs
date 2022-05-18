use std::{
    fmt::{Debug, Display},
    ops::{Bound, Deref},
    str::FromStr,
};

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use smallvec::{SmallVec, ToSmallVec};

use super::{Error, Result};

/// An index key.
///
/// The optimized no-alloc storage size is 16 bytes, which is the size of a
/// UUID. This is not a coincidence.
#[derive(Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct IndexKey(SmallVec<[u8; 16]>);

/// A display format for index keys.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum IndexKeyDisplayFormat {
    #[serde(rename = "hex")]
    Hex,
    #[serde(rename = "utf8")]
    Utf8,
}

impl FromStr for IndexKeyDisplayFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hex" => Ok(Self::Hex),
            "utf8" => Ok(Self::Utf8),
            _ => Err(Error::InvalidIndexKeyDisplayFormat(s.to_string())),
        }
    }
}

impl Display for IndexKeyDisplayFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hex => write!(f, "hex"),
            Self::Utf8 => write!(f, "utf8"),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CompositeIndexKey {
    #[serde(rename = "f")]
    first: IndexKey,
    #[serde(rename = "s")]
    second: IndexKey,
}

impl IndexKey {
    /// Compose an index key from two other index keys.
    ///
    /// This is useful when dealing with composite indexes.
    pub fn compose(first: impl Into<Self>, second: impl Into<Self>) -> Self {
        rmp_serde::to_vec(&CompositeIndexKey {
            first: first.into(),
            second: second.into(),
        })
        .unwrap()
        .into()
    }

    /// Compose this index key with another index key.
    #[must_use]
    pub fn compose_with(self, other: Self) -> Self {
        Self::compose(self, other)
    }

    /// Decompose an index key into two other index keys.
    ///
    /// # Errors
    ///
    /// If the index key is not a composite index key, `Error::InvalidIndexKey` will be returned.
    pub fn decompose(&self) -> Result<(Self, Self)> {
        rmp_serde::from_slice(&self.0)
            .map_err(|err| {
                Error::InvalidIndexKey(format!("failed to decompose index key: {}", err))
            })
            .map(|CompositeIndexKey { first, second }| (first, second))
    }

    /// Instanciates a new index key from its hexadecimal representation.
    ///
    /// # Errors
    ///
    /// Returns an error if the hexadecimal representation is invalid.
    pub fn from_hex(hex: &str) -> Result<Self> {
        Ok(hex::decode(hex)?.into())
    }

    /// Get an hexadecimal string representation of the index key.
    pub fn to_hex(&self) -> String {
        hex::encode(self)
    }

    /// Interpret the index key as an UTF-8 string.
    ///
    /// # Errors
    ///
    /// Returns an error if the index key is not valid UTF-8.
    pub fn to_utf8_string(&self) -> Result<String> {
        Ok(String::from_utf8(self.to_vec())?)
    }

    /// Interpret the index key as an UTF-8 string or return a default value if
    /// it can't.
    pub fn to_utf8_string_or(&self, f: impl FnOnce() -> String) -> String {
        self.to_utf8_string().unwrap_or_else(|_| f())
    }

    /// Join an index key and a slice of bytes into a new index key.
    #[must_use]
    pub fn join(&self, other: impl AsRef<[u8]>) -> Self {
        let mut bytes = self.0.clone();
        bytes.extend_from_slice(other.as_ref());

        Self(bytes)
    }

    /// Parse the index key string according to the specified format.
    ///
    /// # Errors
    ///
    /// Returns an error if the index key string is not valid according to the
    /// specified format.
    pub fn parse(format: IndexKeyDisplayFormat, s: &str) -> Result<Self> {
        match format {
            IndexKeyDisplayFormat::Hex => Self::from_hex(s),
            IndexKeyDisplayFormat::Utf8 => Ok(s.into()),
        }
    }

    /// Format the index key according to the specified format.
    pub fn format(&self, format: IndexKeyDisplayFormat) -> String {
        match format {
            IndexKeyDisplayFormat::Hex => self.to_hex(),
            IndexKeyDisplayFormat::Utf8 => self
                .to_utf8_string()
                .unwrap_or_else(|_| "<invalid utf-8>".to_string()),
        }
    }

    // Check if the index key has the specified prefix.
    pub fn has_prefix(&self, other: &Self) -> bool {
        if self.len() < other.len() {
            false
        } else {
            &self.0[..other.0.len()] == other.0.as_slice()
        }
    }
}

impl Debug for IndexKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const N: usize = 2;
        let repr = self
            .to_hex()
            .chars()
            .enumerate()
            .flat_map(|(i, c)| {
                if i != 0 && i % N == 0 {
                    Some(' ')
                } else {
                    None
                }
                .into_iter()
                .chain(std::iter::once(c))
            })
            .collect::<String>();

        write!(f, "{}", repr)
    }
}

impl Deref for IndexKey {
    type Target = SmallVec<[u8; 16]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for IndexKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<SmallVec<[u8; 16]>> for IndexKey {
    fn from(bytes: SmallVec<[u8; 16]>) -> Self {
        Self(bytes)
    }
}

impl From<&[u8]> for IndexKey {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_smallvec())
    }
}

impl From<Vec<u8>> for IndexKey {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes.to_smallvec())
    }
}

impl From<u8> for IndexKey {
    fn from(v: u8) -> Self {
        Self([v].to_smallvec())
    }
}

impl From<u16> for IndexKey {
    fn from(v: u16) -> Self {
        let mut buf = SmallVec::<[u8; 16]>::new();
        buf.resize(2, 0);
        byteorder::BigEndian::write_u16(&mut buf, v);
        Self(buf)
    }
}

impl From<i16> for IndexKey {
    fn from(v: i16) -> Self {
        let mut buf = SmallVec::<[u8; 16]>::new();
        buf.resize(2, 0);
        byteorder::BigEndian::write_i16(&mut buf, v);
        Self(buf)
    }
}

impl From<u32> for IndexKey {
    fn from(v: u32) -> Self {
        let mut buf = SmallVec::<[u8; 16]>::new();
        buf.resize(4, 0);
        byteorder::BigEndian::write_u32(&mut buf, v);
        Self(buf)
    }
}

impl From<i32> for IndexKey {
    fn from(v: i32) -> Self {
        let mut buf = SmallVec::<[u8; 16]>::new();
        buf.resize(4, 0);
        byteorder::BigEndian::write_i32(&mut buf, v);
        Self(buf)
    }
}

impl From<u64> for IndexKey {
    fn from(v: u64) -> Self {
        let mut buf = SmallVec::<[u8; 16]>::new();
        buf.resize(8, 0);
        byteorder::BigEndian::write_u64(&mut buf, v);
        Self(buf)
    }
}

impl From<i64> for IndexKey {
    fn from(v: i64) -> Self {
        let mut buf = SmallVec::<[u8; 16]>::new();
        buf.resize(8, 0);
        byteorder::BigEndian::write_i64(&mut buf, v);
        Self(buf)
    }
}

impl From<u128> for IndexKey {
    fn from(v: u128) -> Self {
        let mut buf = SmallVec::<[u8; 16]>::new();
        buf.resize(16, 0);
        byteorder::BigEndian::write_u128(&mut buf, v);
        Self(buf)
    }
}

impl From<i128> for IndexKey {
    fn from(v: i128) -> Self {
        let mut buf = SmallVec::<[u8; 16]>::new();
        buf.resize(16, 0);
        byteorder::BigEndian::write_i128(&mut buf, v);
        Self(buf)
    }
}

impl From<String> for IndexKey {
    fn from(v: String) -> Self {
        Self(v.as_bytes().to_smallvec())
    }
}

impl From<&str> for IndexKey {
    fn from(v: &str) -> Self {
        Self(v.as_bytes().to_smallvec())
    }
}

pub trait IndexKeyBound {
    fn as_index_key_bound(&self) -> Bound<IndexKey>;
}

impl<T: Into<IndexKey> + Clone> IndexKeyBound for Bound<&T> {
    fn as_index_key_bound(&self) -> Bound<IndexKey> {
        match &self {
            Bound::Included(v) => Bound::Included((*v).clone().into()),
            Bound::Excluded(v) => Bound::Excluded((*v).clone().into()),
            Bound::Unbounded => Bound::Unbounded,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    trait IntoIndexKey {
        fn into_index_key(self) -> IndexKey;
    }

    impl<T: Into<IndexKey>> IntoIndexKey for T {
        fn into_index_key(self) -> IndexKey {
            self.into()
        }
    }

    #[test]
    fn test_index_key_from() {
        assert_eq!([0x01].into_index_key(), 0x01_u8.into());
        assert_eq!([0x01, 0x00].into_index_key(), 0x100_u16.into());
        assert_eq!([0x01, 0x00].into_index_key(), 0x100_i16.into());
        assert_eq!(
            [0x01, 0x00, 0x00, 0x00].into_index_key(),
            0x1000000_u32.into()
        );
        assert_eq!(
            [0x01, 0x00, 0x00, 0x00].into_index_key(),
            0x1000000_i32.into()
        );
        assert_eq!(
            [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00].into_index_key(),
            0x100000000000000_u64.into()
        );
        assert_eq!(
            [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00].into_index_key(),
            0x100000000000000_i64.into()
        );
        assert_eq!(
            [
                0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00
            ]
            .into_index_key(),
            0x1000000000000000000000000000000_u128.into()
        );
        assert_eq!(
            [
                0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00
            ]
            .into_index_key(),
            0x1000000000000000000000000000000_i128.into()
        );
        assert_eq!(b"hello".into_index_key(), "hello".into());
        assert_eq!(b"hello".into_index_key(), "hello".to_string().into());
        assert_eq!(
            b"hello".into_index_key().to_utf8_string().unwrap(),
            "hello".to_string(),
        );
    }

    #[test]
    fn test_composite_index_key() {
        let first = "first".into_index_key();
        let second = "second".into_index_key();
        let key = first.compose_with(second);

        let (first, second) = key.decompose().unwrap();

        assert_eq!(first, "first".into_index_key());
        assert_eq!(second, "second".into_index_key());
    }
}
