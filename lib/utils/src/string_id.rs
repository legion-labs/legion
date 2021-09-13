//! Hashed string identifiers.
//!
//! Strings work well in many use cases because of their human-readable nature.
//! The downside of using strings is that they take significant amount
//! of memory and that comparison of two strings is costly.
//!
//! This module provides a hashed string representation that can be compared efficiently
//! and provides a debugging readability.
//!
//! # Example
//!
//! ```
//! # use legion_utils::string_id::StringId;
//! let sid = StringId::new("world");
//! println!("Hello {}", StringId::lookup_name(sid).unwrap());
//! ```

use lazy_static::lazy_static;
use std::{collections::HashMap, sync::Mutex};

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
    pub fn new(name: &str) -> Self {
        let id = Self::compute_sid(name);
        let out = DICTIONARY.lock().unwrap().insert(id, name.to_owned());
        assert!(out.is_none() || out.unwrap() == name);
        id
    }

    /// Returns a string that is the source of sid.
    ///
    /// None is returned if such a string is unknown. This can happen when the dictionary is disabled or
    /// provided `StringId` was created using [`Self::from_raw`].
    pub fn lookup_name(sid: Self) -> Option<String> {
        DICTIONARY.lock().unwrap().get(&sid).cloned()
    }

    const fn compute_sid(name: &str) -> Self {
        let v = Self::CRC32_ALGO.checksum(name.as_bytes());
        Self(v)
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::StringId;

    #[test]
    fn static_string() {
        let raw = StringId::from_raw(2357529937); // "hello world"

        assert!(StringId::lookup_name(raw).is_none());

        let sid = StringId::new("hello world");
        assert_eq!(StringId::lookup_name(sid).unwrap().as_str(), "hello world");

        assert_eq!(StringId::lookup_name(raw).unwrap().as_str(), "hello world");
    }

    #[test]
    fn dynamic_string() {
        let string = format!("{:?}", SystemTime::now());

        let sid = StringId::new(string.as_str());
        assert_eq!(
            StringId::lookup_name(sid).unwrap().as_str(),
            string.as_str()
        );
    }
}
