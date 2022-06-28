use std::{fmt, hash::Hash, ops::Add, str::FromStr};

use serde::{Deserialize, Serialize};

use lgn_data_runtime::ResourceTypeAndId;

/// Identifier of a resource.
///
/// Resources are identified in a path-like manner.
/// All `ResourcePathName` instances start with a separator **/**.
/// Each consecutive separator represents a directory while the component
/// after the last separator is the display name of the resource.
///
/// # Example
/// ```
/// # use lgn_data_offline::ResourcePathName;
/// let mut path = ResourcePathName::new("model");
/// path.push("npc");
/// path.push("dragon");
///
/// assert_eq!(path.to_string(), "/model/npc/dragon");
/// ```
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash, Default)]
pub struct ResourcePathName(String);

const SEPARATOR: char = '/';

impl ResourcePathName {
    /// New `ResourcePathName` in root directory.
    ///
    /// # Panics:
    ///
    /// Panics if name starts with a separator (is an absolute path).
    pub fn new(name: impl AsRef<str>) -> Self {
        let mut s = if !name.as_ref().starts_with(SEPARATOR) {
            String::from(SEPARATOR)
        } else {
            String::new()
        };
        s.push_str(name.as_ref());
        Self(s)
    }

    /// Extends self with path.
    ///
    /// # Panics:
    ///
    /// Panics if path starts with a separator (is an absolute path).
    pub fn push(&mut self, path: impl AsRef<str>) {
        self.0.push(SEPARATOR);
        self.0.push_str(path.as_ref());
    }

    /// Return the `ResourcePathName` as a str
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Extract the parenting info from the resource name
    pub fn extract_parent_info(&self) -> (Option<ResourceTypeAndId>, &str) {
        if let Some((resource_id, relative_name)) = self
            .0
            .as_str()
            .strip_prefix("/!")
            .and_then(|v| v.split_once('/'))
        {
            if let Ok(resource_id) = ResourceTypeAndId::from_str(resource_id) {
                return (Some(resource_id), relative_name);
            }
        }
        (None, &self.0)
    }

    /// Replace the encoded parent identifier
    pub fn replace_parent_info(
        &mut self,
        new_parent_id: Option<ResourceTypeAndId>,
        new_path: Option<Self>,
    ) {
        if let Some((resource_id, relative_name)) = self
            .0
            .as_str()
            .strip_prefix("/!")
            .and_then(|v| v.split_once('/'))
        {
            let resource_id = new_parent_id.map_or(resource_id.into(), |s| s.to_string());
            let new_path = new_path.as_ref().map_or(
                format!("/{}", relative_name),
                std::string::ToString::to_string,
            );
            self.0 = format!("/!{}{}", resource_id, new_path);
        } else if let Some(new_parent_id) = new_parent_id {
            let relative_name = self.0.as_str().rsplit('/').next().unwrap_or("");
            self.0 = format!(
                "/!{}{}",
                new_parent_id,
                new_path.unwrap_or_else(|| relative_name.into())
            );
        } else if let Some(new_path) = new_path {
            *self = new_path;
        }
    }
}

lgn_data_model::implement_primitive_type_def!(ResourcePathName);

impl Add<&'_ str> for ResourcePathName {
    type Output = Self;

    fn add(self, rhs: &'_ str) -> Self::Output {
        (self.0 + rhs).into()
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
        Self(s)
    }
}

impl FromStr for ResourcePathName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().next() {
            Some(c) => {
                if c != SEPARATOR {
                    return Err(());
                }
            }
            None => return Err(()),
        }
        Ok(Self(s.to_owned()))
    }
}

impl From<&str> for ResourcePathName {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl<T: AsRef<str>> From<&T> for ResourcePathName {
    fn from(s: &T) -> Self {
        Self::new(s.as_ref())
    }
}

impl AsRef<str> for ResourcePathName {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}
