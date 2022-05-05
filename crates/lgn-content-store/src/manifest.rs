use std::{fmt::Debug, io::Write};

use lgn_tracing::{debug, span_fn};
use thiserror::Error as TError;
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::{Identifier, InvalidIdentifier, Result};

/// An error type for the content-store crate.
#[derive(TError, Debug)]
pub enum InvalidManifest {
    #[error("unknown format: {0:x}")]
    UnknownFormat(u8),
    #[error("invalid identifier: {0}")]
    InvalidIdentifier(#[from] InvalidIdentifier),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum ManifestFormat {
    Linear = 1,
}

impl TryFrom<u8> for ManifestFormat {
    type Error = InvalidManifest;

    fn try_from(value: u8) -> Result<Self, InvalidManifest> {
        match value {
            1 => Ok(Self::Linear),
            _ => Err(InvalidManifest::UnknownFormat(value)),
        }
    }
}

/// A index of chunks.
pub enum Manifest {
    Linear(Vec<Identifier>),
}

impl Manifest {
    /// Read a manifest from the given reader.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is too small or the format is unknown.
    #[span_fn]
    pub async fn read_from(mut r: impl AsyncRead + Unpin) -> Result<Self, InvalidManifest> {
        debug!("Manifest::read_from()");

        let mut buf = [0u8; 1];

        r.read_exact(&mut buf).await?;

        match ManifestFormat::try_from(buf[0])? {
            ManifestFormat::Linear => {
                // Now we must read a list of identifiers, each prefixed with their size.
                let mut identifiers = Vec::new();
                let mut id_buf = [0u8; 256];

                loop {
                    let id_size = match r.read_exact(&mut buf).await {
                        Ok(_) => buf[0] as usize,
                        Err(err) => {
                            if err.kind() == std::io::ErrorKind::UnexpectedEof {
                                break;
                            }

                            return Err(err.into());
                        }
                    };

                    r.read_exact(&mut id_buf[..id_size]).await?;

                    identifiers.push(Identifier::read_from(std::io::Cursor::new(
                        &id_buf[..id_size],
                    ))?);
                }

                Ok(Self::Linear(identifiers))
            }
        }
    }

    /// Write the chunk index to the specified buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if the write fails.
    pub fn write_all_to(&self, mut w: impl Write) -> std::io::Result<()> {
        w.write_all(&[self.format() as u8])?;

        match self {
            Self::Linear(identifiers) => {
                for id in identifiers {
                    let id_size = id.bytes_len();
                    w.write_all(&[id_size as u8])?;

                    id.write_to(&mut w)?;
                }

                Ok(())
            }
        }
    }

    fn format(&self) -> ManifestFormat {
        match self {
            Manifest::Linear(_) => ManifestFormat::Linear,
        }
    }

    /// Returns the list of identifiers listed in the manifest.
    pub fn identifiers(&self) -> &Vec<Identifier> {
        match self {
            Manifest::Linear(ids) => ids,
        }
    }
}
