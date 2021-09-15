//! Hashed string identifiers.
//!
//! > -----
//! > # *WORK IN PROGRESS*
//! >
//! > `StringId` is under development. The remaining issue are:
//! > * [Feature unification](https://doc.rust-lang.org/cargo/reference/features.html#feature-unification) poses a problem of being able to compile the engine without *stringid-debug* feature.
//! > * const `StringId` does not add the string name to the dictionary significantly limiting the usability of `StringId`.
//! > * Performance of `DICTIONARY` needs to be evaluated with emphasis on [`lookup_debug_name`]
//! > * Current implementation of [`lookup_debug_name`] allocates a String while it would be better if it returned a `&str`.
//! >
//! > Helpful resources around the topic include:
//! > * [Hash algorithm comparison](https://softwareengineering.stackexchange.com/questions/49550/which-hashing-algorithm-is-best-for-uniqueness-and-speed/145633#145633)
//! > * [Hash collision probabilities](https://preshing.com/20110504/hash-collision-probabilities/)
//! > -----
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
//! println!("Hello {}", sid.debug_name().unwrap());
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
//! println!("Hello {}", sid.debug_name().unwrap());
//! ```

#[cfg(feature = "stringid-debug")]
use lazy_static::lazy_static;
use std::fmt;
#[cfg(feature = "stringid-debug")]
use std::{collections::HashMap, sync::RwLock};

#[cfg(feature = "stringid-debug")]
lazy_static! {
    static ref DICTIONARY: RwLock<HashMap<StringId, String>> = RwLock::new(HashMap::<_, _>::new());
}

const CRC32_ALGO: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_CKSUM);

/// Hashed string representation.
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct StringId(u32);

impl StringId {
    /// Creates a `StringId` from a raw integer value.
    ///
    /// This potentially results in a `StringId` without a string representation.
    /// For such a `StringId` [`Self::debug_name`] can return None.
    pub const fn from_raw(id: u32) -> Self {
        Self(id)
    }

    /// Creates a new `StringId` from a string and adds that string to a dictionary for later lookup.
    ///
    /// `sid!` macro should be preferred as it computes `StringId` at compile time.
    pub fn compute_new(name: &str) -> Self {
        let id = compute_sid(name);
        insert_debug_name(id, name);
        id
    }

    /// Returns a string that is the source of sid.
    ///
    /// None is returned if such a string is unknown. This can happen when the dictionary is disabled or
    /// provided `StringId` was created using [`Self::from_raw`].
    pub fn debug_name(&self) -> Option<String> {
        lookup_debug_name(self)
    }
}

impl fmt::Debug for StringId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("StringId")
            .field(&self.0)
            .field(&self.debug_name())
            .finish()
    }
}

/// Inserts (id, name) tuple into the dictionary.
pub fn insert_debug_name(id: StringId, name: &str) {
    #[cfg(feature = "stringid-debug")]
    {
        let out = DICTIONARY.write().unwrap().insert(id, name.to_owned());
        assert!(out.is_none() || out.unwrap() == name);
    }

    #[cfg(not(feature = "stringid-debug"))]
    {
        let _id = id;
        let _name = name;
    }
}

/// Returns a string behind the `StringId` if one is known, None otherwise.
pub fn lookup_debug_name(sid: &StringId) -> Option<String> {
    #[cfg(feature = "stringid-debug")]
    return DICTIONARY.read().unwrap().get(sid).cloned();

    #[cfg(not(feature = "stringid-debug"))]
    return Some(format!("{}", sid.0));
}

/// Returns `StringId` representation of name without adding `name` to the dictionary.
pub const fn compute_sid(name: &str) -> StringId {
    StringId::from_raw(CRC32_ALGO.checksum(name.as_bytes()))
}

/// Computes `StringId` value at compile time and adds that string to a dictionary for later lookup.
#[macro_export]
macro_rules! sid {
    ($s:expr) => {{
        const SID: $crate::string_id::StringId = $crate::string_id::compute_sid($s);
        $crate::string_id::insert_debug_name(SID, $s);
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
        assert_eq!(sid.debug_name().unwrap().as_str(), "hello");
    }

    #[test]
    #[cfg(feature = "stringid-debug")]
    fn runtime_dict() {
        let raw = StringId::from_raw(4271552933); // "foo"

        assert!(raw.debug_name().is_none());

        let sid = StringId::compute_new("foo");
        assert_eq!(sid.debug_name().unwrap().as_str(), "foo");
        assert_eq!(sid, raw);
    }

    #[test]
    #[cfg(not(feature = "stringid-debug"))]
    fn no_dict() {
        let raw = StringId::from_raw(3954879214); // "bar"
        assert_eq!(raw.debug_name().unwrap().as_str(), "3954879214");
    }
}
