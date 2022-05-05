use byteorder::ReadBytesExt;
use serde::{de::Visitor, Deserialize, Serialize};
use smallvec::SmallVec;
use std::{
    fmt::{Debug, Display, Formatter},
    io::{Read, Write},
    str::FromStr,
};
use thiserror::Error as TError;

use super::Result;
use crate::{
    buf_utils::{get_size_len, read_prefixed_size, write_prefixed_size},
    Identifier,
};

/// An error type for the content-store crate.
#[derive(TError, Debug)]
pub enum InvalidHashRef {
    #[error("unknown hash algorithm: {0:x}")]
    UnknownHashAlgorithm(u8),
    #[error("missing size")]
    MissingSize,
    #[error("missing data")]
    MissingData,
    #[error("cannot parse an alias as a hash-reference")]
    CannotParseAlias,
    #[error("cannot parse a manifest as a hash-reference")]
    CannotParseManifest,
    #[error("invalid high-bits: {0:02x}")]
    InvalidHighBits(u8),
    #[error("base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// A content-store hash reference.
///
/// A hash refererence contains the size and hash of a blob of data, and is used
/// as a low-level identifier to store and retrieve the data.
///
/// Hash references are unique to the data they represent and can be strictly
/// compared to one another for equality.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HashRef {
    pub size: u64,
    pub alg: HashAlgorithm,
    pub hash: SmallVec<[u8; Self::HASH_SIZE]>,
}

impl Debug for HashRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.to_string())
    }
}

/// A hash algorithm used to compute the identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum HashAlgorithm {
    Blake3 = 1,
}

impl Display for HashAlgorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HashAlgorithm::Blake3 => write!(f, "blake3"),
        }
    }
}

impl TryFrom<u8> for HashAlgorithm {
    type Error = InvalidHashRef;

    fn try_from(value: u8) -> Result<Self, InvalidHashRef> {
        match value {
            1 => Ok(Self::Blake3),
            _ => Err(InvalidHashRef::UnknownHashAlgorithm(value)),
        }
    }
}

impl Display for HashRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut enc = base64::write::EncoderStringWriter::new(base64::URL_SAFE_NO_PAD);

        self.write_to(&mut enc).unwrap();

        write!(f, "{}", enc.into_inner())
    }
}

impl FromStr for HashRef {
    type Err = InvalidHashRef;

    fn from_str(s: &str) -> Result<Self, InvalidHashRef> {
        let buf = match base64::decode_config(s, base64::URL_SAFE_NO_PAD) {
            Ok(buf) => buf,
            Err(err) => return Err(err.into()),
        };

        Self::read_from(std::io::Cursor::new(buf))
    }
}

impl HashRef {
    pub(crate) const HASH_SIZE: usize = 32;

    /// Create a new hash ref from a hash to a blob and its associated size
    ///
    /// The hash ref will contain a reference to the blob.
    pub(crate) fn new(size: u64, alg: HashAlgorithm, hash: &[u8]) -> Self {
        Self {
            size,
            alg,
            hash: hash.into(),
        }
    }

    /// Create a new hash ref by hashing the specified data.
    ///
    /// The hash ref will contain a reference to the blob.
    pub(crate) fn new_from_data(data: &[u8]) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(data);
        let hash = hasher.finalize();

        let alg = HashAlgorithm::Blake3;

        Self::new(
            data.len().try_into().expect("size does not fit in usize"),
            alg,
            hash.as_bytes(),
        )
    }

    /// Return the size of the data referenced by this hash ref.
    pub fn data_size(&self) -> usize {
        self.size.try_into().expect("size does not fit in usize")
    }

    /// Read a hash ref from a reader.
    ///
    /// # Errors
    ///
    /// If the identifier is not valid, `Error::InvalidIdentifier` is returned.
    pub fn read_from(mut r: impl Read) -> Result<Self, InvalidHashRef> {
        // If you modify this function, you must also modify the corresponding
        // one in `Identifier`.

        Ok(match read_prefixed_size(&mut r)? {
            (None, high_bits) => match high_bits {
                Identifier::HIGH_BITS_DATA => return Err(InvalidHashRef::MissingSize),
                Identifier::HIGH_BITS_ALIAS => return Err(InvalidHashRef::CannotParseAlias),
                high_bits => return Err(InvalidHashRef::InvalidHighBits(high_bits)),
            },
            (Some(size), high_bits) => match high_bits {
                Identifier::HIGH_BITS_HASH_REF => {
                    let alg = r.read_u8()?.try_into()?;

                    let mut data = Vec::with_capacity(Self::HASH_SIZE);
                    r.read_to_end(&mut data)?;

                    if data.is_empty() {
                        return Err(InvalidHashRef::MissingData);
                    }

                    Self {
                        size,
                        alg,
                        hash: data.into(),
                    }
                }
                Identifier::HIGH_BITS_MANIFEST_REF => {
                    return Err(InvalidHashRef::CannotParseManifest)
                }
                high_bits => return Err(InvalidHashRef::InvalidHighBits(high_bits)),
            },
        })
    }

    /// Write this identifier to the specified writer.
    ///
    /// # Errors
    ///
    /// Returns an error if `w` cannot be written to.
    pub fn write_to(&self, mut w: impl Write) -> std::io::Result<()> {
        write_prefixed_size(&mut w, self.size, Identifier::HIGH_BITS_HASH_REF)?;
        w.write_all(&[self.alg as u8])?;
        w.write_all(&self.hash)?;

        Ok(())
    }

    /// Create a vector from this identifier.
    pub fn as_vec(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.bytes_len());
        self.write_to(&mut buf).unwrap();
        buf
    }

    /// Returns the size of this identifier, when serialized as a byte vector.
    pub fn bytes_len(&self) -> usize {
        get_size_len(self.size) + 2 + self.hash.len()
    }
}

impl Serialize for HashRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            let id = self.to_string();
            serializer.serialize_str(&id)
        } else {
            serializer.serialize_bytes(&self.as_vec())
        }
    }
}

struct HashRefVisitor;

impl<'de> Visitor<'de> for HashRefVisitor {
    type Value = HashRef;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        HashRef::read_from(std::io::Cursor::new(v.to_vec()))
            .map_err(|err| serde::de::Error::custom(err.to_string()))
    }
}

impl<'de> Deserialize<'de> for HashRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let id = String::deserialize(deserializer)?;
            Ok(Self::from_str(&id).unwrap())
        } else {
            deserializer.deserialize_bytes(HashRefVisitor)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_ref_from_str() {
        assert_eq!(
            HashRef::new(
                2,
                HashAlgorithm::Blake3,
                &[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]
            ),
            "AQIBCgsMDQ4P".parse().unwrap()
        );
        assert_eq!(
            HashRef::new(
                256,
                HashAlgorithm::Blake3,
                &[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]
            ),
            "AgEAAQoLDA0ODw".parse().unwrap()
        );

        // Empty identifier.
        assert!("".parse::<HashRef>().is_err());

        // Missing Hash Algorithm identifier and hash.
        assert!("AQE".parse::<HashRef>().is_err());

        // Invalid Hash Algorithm identifier.
        assert!("AQEA".parse::<HashRef>().is_err());

        // Missing hash.
        assert!("AQEB".parse::<HashRef>().is_err());
    }

    #[test]
    fn test_hash_ref_to_string() {
        assert_eq!(
            HashRef::new(
                2,
                HashAlgorithm::Blake3,
                &[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]
            )
            .to_string(),
            "AQIBCgsMDQ4P"
        );
        assert_eq!(
            HashRef::new(
                256,
                HashAlgorithm::Blake3,
                &[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]
            )
            .to_string(),
            "AgEAAQoLDA0ODw"
        );
    }
}
