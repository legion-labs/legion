use std::{
    fmt::Display,
    ops::{Bound, Deref},
    str::FromStr,
};

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::{Error, Result};

/// An index key.
///
/// The optimized no-alloc storage size is 16 bytes, which is the size of a
/// UUID. This is not a coincidence.
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct IndexKey(SmallVec<[u8; 16]>);

impl Display for IndexKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl FromStr for IndexKey {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(hex::decode(s)?.into()))
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

pub trait IntoIndexKey {
    fn into_index_key(self) -> IndexKey;
}

impl<T: Into<SmallVec<[u8; 16]>>> IntoIndexKey for T {
    fn into_index_key(self) -> IndexKey {
        IndexKey(self.into())
    }
}

impl From<SmallVec<[u8; 16]>> for IndexKey {
    fn from(bytes: SmallVec<[u8; 16]>) -> Self {
        Self(bytes)
    }
}

impl From<u8> for IndexKey {
    fn from(v: u8) -> Self {
        [v].into_index_key()
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
        v.into_bytes().into_index_key()
    }
}

impl From<&str> for IndexKey {
    fn from(v: &str) -> Self {
        v.as_bytes().into_index_key()
    }
}

impl IndexKey {
    /// Interpret the index key as UTF-8 string.
    ///
    /// # Errors
    ///
    /// If the index key is not valid UTF-8, an error is returned.
    pub fn into_string_key(self) -> Result<String> {
        let v: Vec<u8> = self.0.into_iter().collect();

        String::from_utf8(v).map_err(|err| {
            Error::InvalidIndexKey(format!(
                "index key cannot be converted to UTF-8 string: {}",
                err
            ))
        })
    }

    pub fn from_slice(bytes: &[u8]) -> Self {
        bytes.into_index_key()
    }

    #[must_use]
    pub fn join(&self, other: impl AsRef<[u8]>) -> Self {
        let mut bytes = self.0.clone();
        bytes.extend_from_slice(other.as_ref());

        Self(bytes)
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
            b"hello".into_index_key().into_string_key().unwrap(),
            "hello".to_string(),
        );
    }
}
