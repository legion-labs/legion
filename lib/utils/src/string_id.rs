//! Hashed string identifiers.
//!
//! Strings work well in many use cases because of their human-readable nature.
//! The downside of using strings is that they take significant amount
//! of memory and that comparison of two strings is costly.
//!
//! This module provides a hashed string representation that can be compared efficiently
//! and provides a debugging readability.
//!
//! # Compile time hashing
//!
//! The module provides `sid!` macro which should be the preferred way of creating
//! `StringId`s - it does the hashing at compilation time
//!
//! ```
//! # use legion_utils::{sid, string_id::StringId};
//! let sid = sid!("world");
//! println!("Hello {}", StringId::lookup_name(sid).unwrap());
//! ```
//!
//! # Runtime hashing
//!
//! If the string is not known at compile time [`StringId::compute_new`] can be used to create
//! the `StringId` at runtime.
//!
//! ```
//! # use legion_utils::{sid, string_id::StringId};
//! # let world_input = "world";
//! let sid = StringId::compute_new(world_input);
//! println!("Hello {}", StringId::lookup_name(sid).unwrap());
//! ```

#[cfg(feature = "stringid-debug")]
use lazy_static::lazy_static;
#[cfg(feature = "stringid-debug")]
use std::{collections::HashMap, sync::Mutex};

#[cfg(feature = "stringid-debug")]
lazy_static! {
    static ref DICTIONARY: Mutex<HashMap<StringId, String>> = Mutex::new(HashMap::<_, _>::new());
}

/// Hashed string representation.
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct StringId(u32);

impl StringId {
    const CRC32_ALGO: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_CKSUM);

    /// Creates a `StringId` from a raw integer value.
    ///
    /// This potentially results in a `StringId` without a string representation.
    /// For such a `StringId` [`Self::lookup_name`] can return None.
    pub fn from_raw(id: u32) -> Self {
        Self(id)
    }

    /// Creates a new `StringId` from a string and adds that string to a dictionary for later lookup.
    ///
    /// `sid!` macro should be preferred as it computes `StringId` at compile time.
    pub fn compute_new(name: &str) -> Self {
        let id = Self::compute_sid(name);
        Self::insert_name(id, name);
        id
    }

    /// Returns a string that is the source of sid.
    ///
    /// None is returned if such a string is unknown. This can happen when the dictionary is disabled or
    /// provided `StringId` was created using [`Self::from_raw`].
    #[cfg(feature = "stringid-debug")]
    pub fn lookup_name(sid: Self) -> Option<String> {
        DICTIONARY.lock().unwrap().get(&sid).cloned()
    }

    #[cfg(not(feature = "stringid-debug"))]
    pub fn lookup_name(sid: Self) -> Option<String> {
        Some(format!("{}", sid.0))
    }

    /// Returns `StringId` representation of name without adding `name` to the dictionary.
    pub const fn compute_sid(name: &str) -> Self {
        let v = Self::CRC32_ALGO.checksum(name.as_bytes());
        Self(v)
    }

    /// Inserts (id, name) tuple into the dictionary.
    #[cfg(feature = "stringid-debug")]
    pub fn insert_name(id: Self, name: &str) {
        let out = DICTIONARY.lock().unwrap().insert(id, name.to_owned());
        assert!(out.is_none() || out.unwrap() == name);
    }

    /// Inserts (id, name) tuple into the dictionary.
    #[cfg(not(feature = "stringid-debug"))]
    pub fn insert_name(_id: Self, _name: &str) {}
}

/// Computes `StringId` value at compile time and adds that string to a dictionary for later lookup.
#[macro_export]
macro_rules! sid {
    ($s:expr) => {{
        const SID: StringId = StringId::compute_sid($s);
        StringId::insert_name(SID, $s);
        SID
    }};
}

#[cfg(test)]
mod tests {
    use super::StringId;

    #[test]
    fn basic() {
        let raw = StringId::from_raw(2357529937); // "hello world"
        assert_eq!(raw, sid!("hello world"));
        assert_eq!(raw, StringId::compute_new("hello world"));
    }

    #[test]
    #[cfg(feature = "stringid-debug")]
    fn compile_time_dict() {
        let sid = sid!("hello");
        assert_eq!(StringId::lookup_name(sid).unwrap().as_str(), "hello");
    }

    #[test]
    #[cfg(feature = "stringid-debug")]
    fn runtime_dict() {
        let raw = StringId::from_raw(4271552933); // "foo"

        assert!(StringId::lookup_name(raw).is_none());

        let sid = StringId::compute_new("foo");
        assert_eq!(StringId::lookup_name(sid).unwrap().as_str(), "foo");

        assert_eq!(StringId::lookup_name(raw).unwrap().as_str(), "foo");
    }

    #[test]
    #[cfg(not(feature = "stringid-debug"))]
    fn no_dict() {
        let raw = StringId::from_raw(3954879214); // "bar"
        assert_eq!(StringId::lookup_name(raw).unwrap().as_str(), "3954879214");
    }
}
