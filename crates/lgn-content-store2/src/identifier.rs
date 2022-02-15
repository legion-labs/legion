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
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Identifier {
    HashRef(u64, SmallVec<[u8; HASH_SIZE]>),
    Data(SmallVec<[u8; HASH_SIZE]>),
}

const HASH_SIZE: usize = 32;

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut enc = base64::write::EncoderStringWriter::new(base64::URL_SAFE_NO_PAD);

        match self {
            Self::HashRef(size, hash) => {
                let mut size_buf = [0; 8];
                byteorder::NetworkEndian::write_u64(&mut size_buf, *size);

                let idx = size_buf
                    .iter()
                    .position(|&b| b != 0)
                    .unwrap_or(size_buf.len() - 1);

                let size_len: u8 = (size_buf.len() - idx).try_into().unwrap();

                enc.write_all(&[size_len]).unwrap();
                enc.write_all(&size_buf[idx..]).unwrap();
                enc.write_all(hash).unwrap();
            }
            Self::Data(data) => {
                enc.write_all(&[0_u8]).unwrap();
                enc.write_all(data).unwrap();
            }
        }

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

            Self::HashRef(size, buf[(1 + size_len)..].into())
        })
    }
}

impl Identifier {
    pub(crate) const SIZE_THRESHOLD: usize = HASH_SIZE;

    /// Create an identifier for an empty file.
    pub fn empty() -> Self {
        Self::Data(SmallVec::new())
    }

    /// Create an identifier from a data slice.
    ///
    /// The identifier will contain the specified data.
    pub fn new_data(data: &[u8]) -> Self {
        Self::Data(data.into())
    }

    /// Create an identifier from a hash to a blob and its associated size
    ///
    /// The identifier will contain a reference to the blob.
    pub fn new_hash_ref(size: usize, hash: &[u8]) -> Self {
        let size: u64 = size.try_into().expect("size cannot exceed u64");

        Self::HashRef(size, hash.into())
    }

    /// Create a new hash ref identifier by hashing the specified data.
    ///
    /// The identifier will contain a reference to the blob.
    pub fn new_hash_ref_from_data(data: &[u8]) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(data);
        let hash = hasher.finalize();

        Self::new_hash_ref(data.len(), hash.as_bytes())
    }

    /// Returns the size of the data pointed to by this identifier.
    pub fn data_size(&self) -> u64 {
        match self {
            Self::HashRef(size, _) => (*size),
            Self::Data(data) => data.len().try_into().expect("size cannot exceed usize"),
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
        matches!(self, Self::HashRef(_, _))
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
            Identifier::new_hash_ref(2, &[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]),
            "AQIKCwwNDg8".parse().unwrap()
        );
        assert_eq!(
            Identifier::new_hash_ref(256, &[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]),
            "AgEACgsMDQ4P".parse().unwrap()
        );
    }

    #[test]
    fn test_identifier_to_string() {
        assert_eq!(Identifier::empty().to_string(), "AA");
        assert_eq!(
            Identifier::new_data(&[0x01, 0x02, 0x03]).to_string(),
            "AAECAw"
        );
        assert_eq!(
            Identifier::new_hash_ref(2, &[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]).to_string(),
            "AQIKCwwNDg8"
        );
        assert_eq!(
            Identifier::new_hash_ref(256, &[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]).to_string(),
            "AgEACgsMDQ4P"
        );
    }
}
