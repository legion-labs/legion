use byteorder::ReadBytesExt;
use serde::{de::Visitor, Deserialize, Serialize};
use smallvec::SmallVec;
use std::{
    fmt::{Display, Formatter},
    io::{Read, Write},
    str::FromStr,
};

use crate::{
    buf_utils::{get_size_len, read_prefixed_size, write_prefixed_size},
    Error, Result,
};

/// A content-store identifier.
///
/// Identifiers convey information about the content they point to. In
/// particular, the type is a base64 representation of a hash of the content,
/// prefixed by its size.
///
/// As a special case and for some really small content, the content itself is
/// contained in the identifier, instead of being referenced by a hash.
///
/// Note that if the content is not bigger than a hash would be, it will be
/// stored on the stack.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Identifier {
    HashRef(u64, HashAlgorithm, SmallVec<[u8; HASH_SIZE]>),
    Data(SmallVec<[u8; SMALL_IDENTIFIER_SIZE]>),
}

const HASH_SIZE: usize = 32;
const SMALL_IDENTIFIER_SIZE: usize = 64; // SmallVec has only a finite number of implementations/supported sizes for the backing array.

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
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::Blake3),
            _ => Err(Error::InvalidHashAlgorithm),
        }
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut enc = base64::write::EncoderStringWriter::new(base64::URL_SAFE_NO_PAD);

        self.write_to(&mut enc).unwrap();

        write!(f, "{}", enc.into_inner())
    }
}

impl FromStr for Identifier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let buf = match base64::decode_config(s, base64::URL_SAFE_NO_PAD) {
            Ok(buf) => buf,
            Err(err) => return Err(Error::InvalidIdentifier(err.into())),
        };

        Self::read_from(std::io::Cursor::new(buf))
    }
}

impl Identifier {
    pub const SMALL_IDENTIFIER_SIZE: usize = SMALL_IDENTIFIER_SIZE;

    /// Create an identifier for an empty file.
    pub fn empty() -> Self {
        Self::Data(SmallVec::new())
    }

    /// Create an identifier from a data slice.
    ///
    /// The identifier will use Small Identifier Optimization if the data is
    /// small identifier.
    pub fn new(data: &[u8]) -> Self {
        if data.len() <= SMALL_IDENTIFIER_SIZE {
            Self::new_data(data)
        } else {
            Self::new_hash_ref_from_data(data)
        }
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
    pub(crate) fn new_hash_ref(size: usize, alg: HashAlgorithm, hash: &[u8]) -> Self {
        let size: u64 = size.try_into().expect("size cannot exceed u64");

        Self::HashRef(size, alg, hash.into())
    }

    /// Create a new hash ref identifier by hashing the specified data.
    ///
    /// The identifier will contain a reference to the blob.
    pub(crate) fn new_hash_ref_from_data(data: &[u8]) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(data);
        let hash = hasher.finalize();

        let alg = HashAlgorithm::Blake3;

        Self::new_hash_ref(data.len(), alg, hash.as_bytes())
    }

    /// Returns the size of the data pointed to by this identifier.
    pub fn data_size(&self) -> usize {
        match self {
            Self::HashRef(size, _, _) => (*size).try_into().expect("size cannot fit in usize"), // This should never happen on a modern architecture.
            Self::Data(data) => data.len(),
        }
    }

    /// Returns the hash algorithm used to compute the identifier.
    pub fn hash_algorithm(&self) -> Option<HashAlgorithm> {
        match self {
            Self::HashRef(_, alg, _) => Some(*alg),
            Self::Data(_) => None,
        }
    }

    /// Returns whether the data pointed to by this identifier is empty.
    pub fn is_empty(&self) -> bool {
        self.data_size() == 0
    }

    /// Returns whether the data is contained in the identifier.
    pub fn is_data(&self) -> bool {
        matches!(self, Self::Data(_))
    }

    /// Returns whether the data pointed to by this identifier is a reference.
    pub fn is_hash_ref(&self) -> bool {
        matches!(self, Self::HashRef(_, _, _))
    }

    /// Checks whether the specified data buffer matches the identifier.
    ///
    /// # Errors
    ///
    /// Returns `Error::DataMismatch` if the data does not match the identifier.
    pub fn matches(&self, buf: &[u8]) -> Result<()> {
        match self {
            Self::Data(data) => {
                if buf != data.as_slice() {
                    Err(Error::DataMismatch {
                        reason: "data differs".into(),
                    })
                } else {
                    Ok(())
                }
            }
            Self::HashRef(size, hash_alg, hash) => {
                if buf.len() != *size as usize {
                    Err(Error::DataMismatch {
                        reason: "data size differs".into(),
                    })
                } else {
                    match hash_alg {
                        HashAlgorithm::Blake3 => {
                            let mut hasher = blake3::Hasher::new();
                            hasher.update(buf);
                            let buf_hash = hasher.finalize();

                            if buf_hash.as_bytes() != hash.as_slice() {
                                Err(Error::DataMismatch {
                                    reason: "data hash differs".into(),
                                })
                            } else {
                                Ok(())
                            }
                        }
                    }
                }
            }
        }
    }

    /// Read an identifier from a reader.
    ///
    /// # Errors
    ///
    /// If the identifier is not valid, `Error::InvalidIdentifier` is returned.
    pub fn read_from(mut r: impl Read) -> Result<Self> {
        Ok(
            match read_prefixed_size(&mut r).map_err(|err| Error::InvalidIdentifier(err.into()))? {
                None => {
                    let mut data = Vec::with_capacity(SMALL_IDENTIFIER_SIZE);
                    r.read_to_end(&mut data).map_err(|err| {
                        Error::InvalidIdentifier(anyhow::anyhow!(
                            "failed to read embedded data: {}",
                            err
                        ))
                    })?;

                    Self::Data(data.into())
                }
                Some(size) => {
                    let alg = r
                        .read_u8()
                        .map_err(|err| {
                            Error::InvalidIdentifier(anyhow::anyhow!(
                                "failed to read hash algorithm: {}",
                                err
                            ))
                        })?
                        .try_into()?;

                    let mut data = Vec::with_capacity(HASH_SIZE);
                    r.read_to_end(&mut data).map_err(|err| {
                        Error::InvalidIdentifier(anyhow::anyhow!(
                            "failed to read embedded data: {}",
                            err
                        ))
                    })?;

                    if data.is_empty() {
                        return Err(Error::InvalidIdentifier(anyhow::anyhow!(
                            "hash algorithm {} requires data",
                            alg
                        )));
                    }

                    Self::HashRef(size, alg, data.into())
                }
            },
        )
    }

    /// Create a vector from this identifier.
    ///
    /// # Errors
    ///
    /// Returns an error if `w` cannot be written to.
    pub fn write_to(&self, mut w: impl Write) -> std::io::Result<()> {
        match self {
            Self::HashRef(size, alg, hash) => {
                write_prefixed_size(&mut w, *size)?;
                w.write_all(&[*alg as u8])?;
                w.write_all(hash)?;
            }
            Self::Data(data) => {
                w.write_all(&[0_u8])?;
                w.write_all(data)?;
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
            Self::HashRef(size, _, hash) => get_size_len(*size) + 2 + hash.len(),
            Self::Data(data) => 1 + data.len(),
        }
    }
}

impl Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.as_vec())
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
        deserializer.deserialize_bytes(IdentifierVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier_from_str() {
        assert_eq!(Identifier::empty(), "AA".parse().unwrap());
        assert_eq!(
            Identifier::new_data(&[0x01, 0x02, 0x03]),
            "AAECAw".parse().unwrap()
        );
        assert_eq!(
            Identifier::new_hash_ref(
                2,
                HashAlgorithm::Blake3,
                &[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]
            ),
            "AQIBCgsMDQ4P".parse().unwrap()
        );
        assert_eq!(
            Identifier::new_hash_ref(
                256,
                HashAlgorithm::Blake3,
                &[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]
            ),
            "AgEAAQoLDA0ODw".parse().unwrap()
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
            Identifier::new_hash_ref(
                2,
                HashAlgorithm::Blake3,
                &[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]
            )
            .to_string(),
            "AQIBCgsMDQ4P"
        );
        assert_eq!(
            Identifier::new_hash_ref(
                256,
                HashAlgorithm::Blake3,
                &[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]
            )
            .to_string(),
            "AgEAAQoLDA0ODw"
        );
    }

    #[test]
    fn test_identifier_matches() {
        let id = Identifier::new_data(&[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]);

        assert!(id.matches(&[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]).is_ok());
        assert!(id.matches(&[0x0A, 0x0B, 0x0C, 0x0D, 0x0F, 0x0F]).is_err());
        assert!(id.matches(&[0x0A, 0x0B]).is_err());

        let id = Identifier::new_hash_ref(
            2,
            HashAlgorithm::Blake3,
            hex::decode("983589fda95f1ee2ca6b6f3120f4f9a81cef431e5ad762df3a4473e20aa97a8c")
                .unwrap()
                .as_slice(),
        );

        assert!(id.matches(&[0x0A, 0x0B]).is_ok());
        assert!(id.matches(&[0x0A, 0x0B, 0x0C]).is_err());
        assert!(id.matches(&[0x0A, 0x0C]).is_err());
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
}
