//! A crate responsible for storage of results of data compilation.

// BEGIN - Legion Labs lints v0.2
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::char_lit_as_u8,
    clippy::checked_conversions,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::fallible_impl_from,
    clippy::filter_map_next,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::if_let_mutex,
    clippy::implicit_clone,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_ok_or,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::mut_mut,
    clippy::mutex_integer,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::string_add_assign,
    clippy::string_lit_as_bytes,
    clippy::string_to_string,
    clippy::todo,
    clippy::trait_duplication_in_bounds,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::useless_transmute,
    clippy::use_self,
    clippy::verbose_file_reads,
    clippy::zero_sized_map_values,
    future_incompatible,
    nonstandard_style,
    broken_intra_doc_links,
    private_intra_doc_links,
    missing_crate_level_docs,
    rust_2018_idioms
)]
// END - Legion Labs standard lints v0.2
// crate-specific exceptions:
#![allow()]
#![warn(missing_docs)]

use std::{
    collections::hash_map::DefaultHasher,
    fmt,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

/// Returns the hash of the provided data.
pub fn content_checksum(data: &[u8]) -> i128 {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish() as i128
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
    /// Write asset to the backing storage.
    fn write(&mut self, id: i128, data: &[u8]) -> Option<()>;

    /// Read asset from the backing storage.
    fn read(&self, id: i128) -> Option<Vec<u8>>;

    /// Remove asset from the backing storage.
    fn remove(&mut self, id: i128);

    /// Returns the description of the asset if it exists.
    ///
    /// This default implementation is quite inefficient as it involves reading the asset's
    /// content to calculate its checksum.
    fn exists(&self, id: i128) -> bool {
        self.read(id).is_some()
    }

    /// Stores the asset and validates its validity afterwards.
    ///
    /// This method calls [`write`](#method.write) to store the asset and [`read`](#method.read) afterwards
    /// to perform the validation.
    fn store(&mut self, data: &[u8]) -> Option<i128> {
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

mod hdd_store;
mod ram_store;

pub use hdd_store::*;
pub use ram_store::*;
