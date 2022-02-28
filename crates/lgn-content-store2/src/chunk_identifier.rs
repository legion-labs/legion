use std::{
    fmt::Formatter,
    io::{Read, Write},
    str::FromStr,
};

use crate::{
    buf_utils::{read_prefixed_size, write_prefixed_size},
    Error, Identifier, Result,
};

/// A chunk identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkIdentifier(u64, Identifier);

impl std::fmt::Display for ChunkIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut enc = base64::write::EncoderStringWriter::new(base64::URL_SAFE_NO_PAD);

        self.write_to(&mut enc).unwrap();

        write!(f, "{}", enc.into_inner())
    }
}

impl FromStr for ChunkIdentifier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let buf = match base64::decode_config(s, base64::URL_SAFE_NO_PAD) {
            Ok(buf) => buf,
            Err(err) => return Err(Error::InvalidIdentifier(err.into())),
        };

        Self::read_from(std::io::Cursor::new(buf))
    }
}

impl ChunkIdentifier {
    /// Creates a new chunk identifier.
    pub fn new(data_size: u64, identifier: Identifier) -> Self {
        Self(data_size, identifier)
    }

    /// Returns the size of the data that this chunk represents.
    pub fn data_size(&self) -> usize {
        self.0.try_into().expect("data_size is too large")
    }

    pub fn content_id(&self) -> &Identifier {
        &self.1
    }

    /// Read an identifier from a reader.
    ///
    /// # Errors
    ///
    /// If the identifier is not valid, `Error::InvalidIdentifier` is returned.
    pub fn read_from(mut r: impl Read) -> Result<Self> {
        match read_prefixed_size(&mut r).map_err(|err| Error::InvalidIdentifier(err.into()))? {
            None => Err(Error::InvalidIdentifier(anyhow::anyhow!(
                "missing chunk size prefix"
            ))),
            Some(size) => {
                let id = Identifier::read_from(&mut r)?;
                Ok(Self::new(size, id))
            }
        }
    }

    /// Create a vector from this identifier.
    ///
    /// # Errors
    ///
    /// Returns an error if `w` cannot be written to.
    pub fn write_to(&self, mut w: impl Write) -> std::io::Result<()> {
        write_prefixed_size(&mut w, self.0)?;
        self.1.write_to(w)
    }
}
