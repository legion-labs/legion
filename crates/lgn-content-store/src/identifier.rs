use byteorder::ReadBytesExt;
use serde::{de::Visitor, Deserialize, Serialize};
use smallvec::SmallVec;
use std::{
    fmt::{Debug, Display, Formatter},
    io::{Read, Write},
    str::FromStr,
};
use thiserror::Error as TError;

use crate::{
    buf_utils::{get_size_len, read_prefixed_size, write_prefixed_size},
    Alias, HashRef, InvalidHashRef, Result,
};

/// An error type for the content-store crate.
#[derive(TError, Debug)]
pub enum InvalidIdentifier {
    #[error("invalid hash-reference: {0}")]
    InvalidHashRef(#[from] InvalidHashRef),
    #[error("empty alias key")]
    EmptyAlias,
    #[error("invalid high-bits: {0:02x}")]
    InvalidHighBits(u8),
    #[error("base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// A content-store identifier.
///
/// Identifiers convey information about the content they point to. In
/// particular, the type is a base64 representation of a hash of the content,
/// prefixed by its size.
///
/// As a special case and for some really small content, the content itself is
/// contained in the identifier, instead of being referenced by a hash.
///
/// Finally, in order to allow for fragments reuse, an identifier can also point
/// to a manifest data-blob that describes the fragments of the real data. This
/// allows storage-optimization like chunking, compression, and static
/// procedural generation.
///
/// # Comparison
///
/// While identifiers are comparable, a given content could be represented by
/// many different identifiers.
///
/// This means that two identical identifiers necessarily point to the same data,
/// but two distinct identifiers could still point to the same data.
///
/// Put otherwise, one can only compare identifiers to test for strict equality
/// of the referenced data in an non-bijective way. Don't do it.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Identifier {
    HashRef(HashRef),
    Data(SmallVec<[u8; SMALL_IDENTIFIER_SIZE]>),
    ManifestRef(u64, Box<Self>),
    Alias(Alias),
}

impl Debug for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.to_string())
    }
}

const SMALL_IDENTIFIER_SIZE: usize = 64; // SmallVec has only a finite number of implementations/supported sizes for the backing array.
const KEY_SIZE: usize = 64;

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut enc = base64::write::EncoderStringWriter::new(base64::URL_SAFE_NO_PAD);

        self.write_to(&mut enc).unwrap();

        write!(f, "{}", enc.into_inner())
    }
}

impl FromStr for Identifier {
    type Err = InvalidIdentifier;

    fn from_str(s: &str) -> Result<Self, InvalidIdentifier> {
        let buf = base64::decode_config(s, base64::URL_SAFE_NO_PAD)?;

        Self::read_from(std::io::Cursor::new(buf))
    }
}

impl TryFrom<crate::api::content_store::ContentId> for Identifier {
    type Error = InvalidIdentifier;

    fn try_from(
        content_id: crate::api::content_store::ContentId,
    ) -> Result<Self, InvalidIdentifier> {
        Self::from_str(&content_id.0)
    }
}

impl From<&Identifier> for crate::api::content_store::ContentId {
    fn from(id: &Identifier) -> Self {
        Self(id.to_string())
    }
}

impl Identifier {
    pub const SMALL_IDENTIFIER_SIZE: usize = SMALL_IDENTIFIER_SIZE;

    pub(crate) const HIGH_BITS_DATA: u8 = 0x00;
    pub(crate) const HIGH_BITS_HASH_REF: u8 = 0x00;
    pub(crate) const HIGH_BITS_ALIAS: u8 = 0x01;
    pub(crate) const HIGH_BITS_MANIFEST_REF: u8 = 0x01;

    /// Create an identifier for an empty file.
    pub fn empty() -> Self {
        Self::Data(SmallVec::new())
    }

    /// Create an identifier from a data slice.
    ///
    /// The identifier will contain the specified data.
    pub(crate) fn new_data(data: &[u8]) -> Self {
        Self::Data(data.into())
    }

    /// Create an identifier from a hash to a blob and its associated size
    ///
    /// The identifier will contain a reference to the blob.
    pub(crate) fn new_hash_ref(hash_ref: HashRef) -> Self {
        Self::HashRef(hash_ref)
    }

    /// Create an identifier from a size and the identifier of a manifest that
    /// describes it.
    ///
    /// The identifier will contain a reference to the manifest.
    pub(crate) fn new_manifest_ref(size: usize, id: Self) -> Self {
        let size: u64 = size.try_into().expect("size cannot exceed u64");

        Self::ManifestRef(size, Box::new(id))
    }

    /// Create an identifier from an alias key.
    pub(crate) fn new_alias(key: Alias) -> Self {
        Self::Alias(key)
    }

    /// Returns whether the data is contained in the identifier.
    pub fn is_data(&self) -> bool {
        matches!(self, Self::Data(_))
    }

    /// Returns whether the data pointed to by this identifier is a reference.
    pub fn is_hash_ref(&self) -> bool {
        matches!(self, Self::HashRef(_))
    }

    /// Returns whether the data pointed to by this identifier is a manifest.
    pub fn is_manifest_ref(&self) -> bool {
        matches!(self, Self::ManifestRef(_, _))
    }

    /// Returns whether the data pointed to by this identifier is an alias.
    pub fn is_alias(&self) -> bool {
        matches!(self, Self::Alias(_))
    }

    /// Read an identifier from a reader.
    ///
    /// # Errors
    ///
    /// If the identifier is not valid, `Error::InvalidIdentifier` is returned.
    pub fn read_from(mut r: impl Read) -> Result<Self, InvalidIdentifier> {
        Ok(match read_prefixed_size(&mut r)? {
            (None, high_bits) => match high_bits {
                Self::HIGH_BITS_DATA => {
                    let mut data = Vec::with_capacity(SMALL_IDENTIFIER_SIZE);
                    r.read_to_end(&mut data)?;

                    Self::Data(data.into())
                }
                Self::HIGH_BITS_ALIAS => {
                    let mut key = Vec::with_capacity(KEY_SIZE);
                    r.read_to_end(&mut key)?;

                    if key.is_empty() {
                        return Err(InvalidIdentifier::EmptyAlias);
                    }

                    Self::Alias(key.into())
                }
                high_bits => return Err(InvalidIdentifier::InvalidHighBits(high_bits)),
            },
            (Some(size), high_bits) => match high_bits {
                Self::HIGH_BITS_HASH_REF => {
                    Self::HashRef(|| -> Result<HashRef, InvalidHashRef> {
                        let alg = r.read_u8()?.try_into()?;

                        let mut data = Vec::with_capacity(HashRef::HASH_SIZE);
                        r.read_to_end(&mut data)?;

                        if data.is_empty() {
                            return Err(InvalidHashRef::MissingData);
                        }

                        Ok(HashRef::new(size, alg, &data))
                    }()?)
                }
                Self::HIGH_BITS_MANIFEST_REF => {
                    let id = Self::read_from(r)?;

                    Self::ManifestRef(size, Box::new(id))
                }
                high_bits => return Err(InvalidIdentifier::InvalidHighBits(high_bits)),
            },
        })
    }

    /// Write this identifier to the specified writer.
    ///
    /// # Errors
    ///
    /// Returns an error if `w` cannot be written to.
    pub fn write_to(&self, mut w: impl Write) -> std::io::Result<()> {
        match self {
            Self::HashRef(hash_ref) => {
                hash_ref.write_to(w)?;
            }
            Self::Data(data) => {
                write_prefixed_size(&mut w, 0, Self::HIGH_BITS_DATA)?;
                w.write_all(data)?;
            }
            Self::ManifestRef(size, id) => {
                write_prefixed_size(&mut w, *size, Self::HIGH_BITS_MANIFEST_REF)?;
                id.write_to(w)?;
            }
            Self::Alias(key) => {
                write_prefixed_size(&mut w, 0, Self::HIGH_BITS_ALIAS)?;
                w.write_all(key)?;
            }
        }

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
        match self {
            Self::HashRef(hash_ref) => hash_ref.bytes_len(),
            Self::Data(data) => 1 + data.len(),
            Self::ManifestRef(size, id) => 1 + get_size_len(*size) + id.bytes_len(),
            Self::Alias(key) => {
                1 + get_size_len(key.len().try_into().expect("can convert len to u64")) + key.len()
            }
        }
    }
}

impl Serialize for Identifier {
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

struct IdentifierVisitor;

impl<'de> Visitor<'de> for IdentifierVisitor {
    type Value = Identifier;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Identifier::read_from(std::io::Cursor::new(v.to_vec()))
            .map_err(|err| serde::de::Error::custom(err.to_string()))
    }
}

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let id = String::deserialize(deserializer)?;
            Ok(Self::from_str(&id).unwrap())
        } else {
            deserializer.deserialize_bytes(IdentifierVisitor)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::HashAlgorithm;

    use super::*;

    #[test]
    fn test_identifier_from_str() {
        assert_eq!(Identifier::empty(), "AA".parse().unwrap());
        assert_eq!(
            Identifier::new_data(&[0x01, 0x02, 0x03]),
            "AAECAw".parse().unwrap()
        );
        assert_eq!(
            Identifier::new_hash_ref(HashRef::new(
                2,
                HashAlgorithm::Blake3,
                &[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]
            )),
            "AQIBCgsMDQ4P".parse().unwrap()
        );
        assert_eq!(
            Identifier::new_hash_ref(HashRef::new(
                256,
                HashAlgorithm::Blake3,
                &[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]
            )),
            "AgEAAQoLDA0ODw".parse().unwrap()
        );
        assert_eq!(
            Identifier::new_manifest_ref(
                42,
                Identifier::new_data(&[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]),
            ),
            "ESoACgsMDQ4P".parse().unwrap()
        );

        // Empty identifier.
        assert!("".parse::<Identifier>().is_err());

        // Missing Hash Algorithm identifier and hash.
        assert!("AQE".parse::<Identifier>().is_err());

        // Invalid Hash Algorithm identifier.
        assert!("AQEA".parse::<Identifier>().is_err());

        // Missing hash.
        assert!("AQEB".parse::<Identifier>().is_err());
    }

    #[test]
    fn test_identifier_to_string() {
        assert_eq!(Identifier::empty().to_string(), "AA");
        assert_eq!(
            Identifier::new_data(&[0x01, 0x02, 0x03]).to_string(),
            "AAECAw"
        );
        assert_eq!(
            Identifier::new_hash_ref(HashRef::new(
                2,
                HashAlgorithm::Blake3,
                &[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]
            ))
            .to_string(),
            "AQIBCgsMDQ4P"
        );
        assert_eq!(
            Identifier::new_hash_ref(HashRef::new(
                256,
                HashAlgorithm::Blake3,
                &[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]
            ))
            .to_string(),
            "AgEAAQoLDA0ODw"
        );
        assert_eq!(
            Identifier::new_manifest_ref(
                42,
                Identifier::new_data(&[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]),
            )
            .to_string(),
            "ESoACgsMDQ4P"
        );
    }

    #[test]
    fn test_identifier_serialization() {
        let id: Identifier = "AAECAw".parse().unwrap();

        assert_eq!(
            rmp_serde::to_vec(&id).unwrap(),
            [0xC4, 0x04, 0x00, 0x01, 0x02, 0x03].to_vec()
        );
        assert_eq!(
            id,
            rmp_serde::from_slice(&[0xC4, 0x04, 0x00, 0x01, 0x02, 0x03]).unwrap()
        );
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    pub struct Message {
        pub id: Identifier,
        pub cas_files: Vec<(String, Identifier)>,
    }

    #[test]
    fn test_identifier_json() {
        let m = Message {
            id: "AAECAw".parse().unwrap(),
            cas_files: vec![("some/path".to_string(), "AAECAw".parse().unwrap())],
        };
        let s = serde_json::to_string_pretty(&m).unwrap();

        let m2: Message = serde_json::from_str(&s).unwrap();
        assert_eq!(m, m2);
    }
}
