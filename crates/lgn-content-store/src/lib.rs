//! A crate responsible for storage of results of data compilation.

// crate-specific lint exceptions:
#![warn(missing_docs)]

use std::{
    fmt,
    hash::Hasher,
    io,
    path::{Path, PathBuf},
};

use lgn_utils::{DefaultHash, DefaultHasher256};

/// Returns the hash of the provided data.
pub fn content_checksum(data: &[u8]) -> Checksum {
    data.default_hash_256().into()
}

/// Returns the hash of the data provided through a Read trait.
///
/// # Errors
///
/// If an error is returned, the checksum is unavailable.
pub fn content_checksum_from_read(data: &mut impl io::Read) -> io::Result<Checksum> {
    let mut hasher = DefaultHasher256::new();
    let mut buffer = [0; 1024];
    loop {
        let count = data.read(&mut buffer)?;
        if count == 0 {
            break;
        }

        hasher.write(&buffer[..count]);
    }

    Ok(hasher.finish_256().into())
}

/// The address of the [`ContentStore`].
///
/// For now, it is equivalent to a `PathBuf` representing a local file path.
/// In the future the address could be representing a remote machine or service.
#[derive(Clone, Debug)]
pub struct ContentStoreAddr(PathBuf);

impl From<&str> for ContentStoreAddr {
    fn from(path: &str) -> Self {
        Self(PathBuf::from(path))
    }
}

impl From<PathBuf> for ContentStoreAddr {
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

impl From<&Path> for ContentStoreAddr {
    fn from(path: &Path) -> Self {
        Self(path.to_owned())
    }
}

impl fmt::Display for ContentStoreAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.0.display()))
    }
}

/// A content-addressable storage interface for dealing with compilation
/// results.
///
/// [`ContentStore`] functions as a *content-addressable storage* - using the
/// [`crate::content_checksum`] function to calculate the checksum of stored
/// content.
// todo: change Option to Error
pub trait ContentStore: Send {
    /// Write content to the backing storage.
    fn write(&mut self, id: Checksum, data: &[u8]) -> Option<()>;

    /// Read content from the backing storage.
    fn read(&self, id: Checksum) -> Option<Vec<u8>>;

    /// Remove content from the backing storage.
    fn remove(&mut self, id: Checksum);

    /// Returns the description of the content if it exists.
    ///
    /// This default implementation is quite inefficient as it involves reading
    /// the content's content to calculate its checksum.
    fn exists(&self, id: Checksum) -> bool {
        self.read(id).is_some()
    }

    /// Stores the content and validates its validity afterwards.
    ///
    /// This method calls [`write`](#method.write) to store the content and
    /// [`read`](#method.read) afterwards to perform the validation.
    fn store(&mut self, data: &[u8]) -> Option<Checksum> {
        let id = content_checksum(data);
        self.write(id, data)?;

        let read = self.read(id)?;

        if id != content_checksum(&read) {
            self.remove(id);
            return None;
        }

        Some(id)
    }
}

mod checksum;
mod hdd_store;
mod ram_store;

pub use checksum::Checksum;
pub use hdd_store::*;
pub use ram_store::*;
