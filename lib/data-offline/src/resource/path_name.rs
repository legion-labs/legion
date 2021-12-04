use std::{fmt, hash::Hash};

use serde::{Deserialize, Serialize};

/// Identifier of a resource.
///
/// Resources are identified in a path-like manner.
/// All `ResourcePathName` instances start with a separator **/**.
/// Each consecutive separator represents a directory while the component
/// after the last separator is the display name of the resource.
///
/// # Example
/// ```
/// # use legion_data_offline::resource::ResourcePathName;
/// let mut path = ResourcePathName::new("model");
/// path.push("npc");
/// path.push("dragon");
///
/// assert_eq!(path.to_string(), "/model/npc/dragon");
/// ```
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
pub struct ResourcePathName(String);

const SEPARATOR: char = '/';

impl ResourcePathName {
    /// New `ResourcePathName` in root directory.
    ///
    /// # Panics:
    ///
    /// Panics if name starts with a separator (is an absolute path).
    pub fn new(name: impl AsRef<str>) -> Self {
        assert_ne!(name.as_ref().chars().next().unwrap(), SEPARATOR);
        let mut s = String::from(SEPARATOR);
        s.push_str(name.as_ref());
        Self(s)
    }

    /// Extends self with path.
    ///
    /// # Panics:
    ///
    /// Panics if path starts with a separator (is an absolute path).
    pub fn push(&mut self, path: impl AsRef<str>) {
        assert_ne!(path.as_ref().chars().next().unwrap(), SEPARATOR);
        self.0.push(SEPARATOR);
        self.0.push_str(path.as_ref());
    }
}

impl fmt::Display for ResourcePathName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[allow(clippy::fallible_impl_from)]
impl From<String> for ResourcePathName {
    fn from(s: String) -> Self {
        assert_eq!(s.chars().next().unwrap(), SEPARATOR);
        Self(s)
    }
}

impl From<&str> for ResourcePathName {
    fn from(s: &str) -> Self {
        Self::from(s.to_owned())
    }
}

impl<T: AsRef<str>> From<&T> for ResourcePathName {
    fn from(s: &T) -> Self {
        Self::from(s.as_ref().to_owned())
    }
}
