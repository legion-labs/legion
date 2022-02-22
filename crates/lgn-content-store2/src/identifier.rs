use byteorder::ByteOrder;
use smallvec::SmallVec;
use std::{fmt::Formatter, io::Write, str::FromStr};

use crate::{Error, Result};

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
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
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

        self.write_all_to(&mut enc).unwrap();

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

        if buf.is_empty() {
            return Err(Error::InvalidIdentifier(anyhow::anyhow!(
                "empty identifier"
            )));
        }

        let size_len = buf[0] as usize;

        Ok(if size_len == 0 {
            // If the size len is 0, the identifier contains the data directly.
            Self::Data(buf[1..].into())
        } else {
            let mut size_buf = [0; 8];

            if size_len > size_buf.len() {
                return Err(Error::InvalidIdentifier(anyhow::anyhow!(
                    "invalid identifier size length"
                )));
            }

            size_buf[8 - size_len..].copy_from_slice(&buf[1..=size_len]);

            let size = byteorder::NetworkEndian::read_u64(&size_buf);

            // We require the identifier to contain a hash algorithm and at
            // least one byte of hash data.
            if buf.len() < size_len + 3 {
                return Err(Error::InvalidIdentifier(anyhow::anyhow!(
                    "invalid identifier length"
                )));
            }

            let alg = buf[size_len + 1].try_into()?;

            Self::HashRef(size, alg, buf[(2 + size_len)..].into())
        })
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
    pub fn data_size(&self) -> u64 {
        match self {
            Self::HashRef(size, _, _) => (*size),
            Self::Data(data) => data.len().try_into().expect("size cannot exceed usize"),
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

    /// Create a vector from this identifier.
    ///
    /// # Errors
    ///
    /// Returns an error if `w` cannot be written to
    pub fn write_all_to(&self, mut w: impl Write) -> std::io::Result<()> {
        match self {
            Self::HashRef(size, alg, hash) => {
                let mut size_buf = [0; 8];
                byteorder::NetworkEndian::write_u64(&mut size_buf, *size);

                let idx = size_buf
                    .iter()
                    .position(|&b| b != 0)
                    .unwrap_or(size_buf.len() - 1);

                let size_len: u8 = (size_buf.len() - idx).try_into().unwrap();

                w.write_all(&[size_len])?;
                w.write_all(&size_buf[idx..])?;
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
        let mut buf = Vec::new();
        self.write_all_to(&mut buf).unwrap();
        buf
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
}
