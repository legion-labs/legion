//! A crate responsible for storage of results of data compilation.

// BEGIN - Legion Labs lints v0.5
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs standard lints v0.5
// crate-specific exceptions:
#![allow()]
#![warn(missing_docs)]

use siphasher::sip128::{self, Hasher128};
use std::hash::Hasher;
use std::{
    fmt,
    hash::Hash,
    io,
    path::{Path, PathBuf},
};

/// Returns the hash of the provided data.
pub fn content_checksum(data: &[u8]) -> Checksum {
    let mut hasher = sip128::SipHasher::new();
    data.hash(&mut hasher);
    hasher.finish128().into()
}

/// Returns the hash of the data provided through a Read trait.
///
/// # Errors
///
/// If an error is returned, the checksum is unavailable.
pub fn content_checksum_from_read(data: &mut impl io::Read) -> io::Result<u128> {
    let mut hasher = sip128::SipHasher::new();
    let mut buffer = [0; 1024];
    loop {
        let count = data.read(&mut buffer)?;
        if count == 0 {
            break;
        }

        hasher.write(&buffer[..count]);
    }

    Ok(hasher.finish128().into())
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

/// A content-addressable storage interface for dealing with compilation results.
///
/// [`ContentStore`] functions as a *content-addressable storage* - using the [`crate::content_checksum`]
/// function to calculate the checksum of stored content.
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
    /// This default implementation is quite inefficient as it involves reading the content's
    /// content to calculate its checksum.
    fn exists(&self, id: Checksum) -> bool {
        self.read(id).is_some()
    }

    /// Stores the content and validates its validity afterwards.
    ///
    /// This method calls [`write`](#method.write) to store the content and [`read`](#method.read) afterwards
    /// to perform the validation.
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
