use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use path_slash::{PathBufExt, PathExt};

use crate::{Error, MapOtherError, Result};

/// Represents a canonical path to a file or directory in the repository.
///
/// A canonical path is a slash-based path relative to the root of the
/// repository that always begins with a slash and never ends with one.
///
/// A canonical path never contains `.` or `..` components and can reliably be
/// compared.
///
/// Canonical paths have strong and stable ordering and comparison guarantees. A
/// file will always compare bigger than one of its direct or indirect parent
/// directories.
///
/// Unrelated directories and files will always respect alphabetical ordering.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct CanonicalPath {
    parts: Vec<String>,
}

impl CanonicalPath {
    pub fn root() -> Self {
        Self { parts: vec![] }
    }

    pub fn new(path: &str) -> Result<Self> {
        let path = path.strip_prefix('/').ok_or_else(|| {
            Error::invalid_canonical_path(path, "canonical paths must start with a `/`")
        })?;

        // Special case for the root.
        if path.is_empty() {
            return Ok(Self { parts: vec![] });
        }

        let parts: Vec<String> = path.split('/').map(&str::to_string).collect();

        if parts.iter().any(std::string::String::is_empty) {
            return Err(Error::invalid_canonical_path(
                path,
                "canonical paths cannot contain empty segments or end with a `/`",
            ));
        }

        Ok(Self { parts })
    }

    pub(crate) fn new_from_name(name: impl Into<String>) -> Self {
        let name: String = name.into();

        if name.is_empty() {
            return Self::root();
        }

        Self { parts: vec![name] }
    }

    /// Create a canonical path from a path relative to a specified root.
    ///
    /// Only the root must be canonical and the path can be either absolute or
    /// relative and will be made canonical. Symlinks are not supported.
    pub async fn new_from_canonical_root(root: &Path, path: &Path) -> Result<Self> {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };

        if path.is_symlink() {
            return Err(Error::symbolic_link_not_supported(path));
        }

        let path = match tokio::fs::canonicalize(&path).await {
            Ok(path) => path,
            Err(err) => {
                return Err(if err.kind() == std::io::ErrorKind::NotFound {
                    Error::unmatched_path(path)
                } else {
                    Error::Other {
                        context: format!("could not canonicalize path `{}`", path.display()),
                        source: err.into(),
                    }
                });
            }
        };

        Self::new_from_canonical_paths(root, &path)
    }

    /// Create a canonical path from a path relative to a specified root.
    ///
    /// The root and path must be canonical paths.
    pub fn new_from_canonical_paths(root: &Path, path: &Path) -> Result<Self> {
        Self::new(&format!(
            "/{}",
            path.strip_prefix(root)
                .map_other_err(format!(
                    "failed to strip prefix `{}` from path `{}`",
                    root.display(),
                    path.display()
                ))?
                .to_slash()
                .ok_or_else(|| Error::Other {
                    context: format!(
                        "failed to make path `{}` relative to root `{}`",
                        path.display(),
                        root.display()
                    ),
                    source: anyhow::anyhow!("path is not relative to root"),
                })?
        ))
    }

    pub fn to_path_buf(&self, root: impl AsRef<Path>) -> PathBuf {
        root.as_ref()
            .join(PathBuf::from_slash(self.parts.join("/")))
    }

    pub fn is_root(&self) -> bool {
        self.parts.is_empty()
    }

    pub fn join(&self, other: &Self) -> Self {
        Self {
            parts: self
                .parts
                .iter()
                .chain(other.parts.iter())
                .cloned()
                .collect(),
        }
    }

    pub fn prepend(&self, part: impl Into<String>) -> Self {
        let mut parts = self.parts.clone();

        let part = part.into();

        if !part.is_empty() {
            parts.insert(0, part);
        }

        Self { parts }
    }

    pub fn append(&self, part: impl Into<String>) -> Self {
        let mut parts = self.parts.clone();

        let part = part.into();

        if !part.is_empty() {
            parts.push(part);
        }

        Self { parts }
    }

    pub fn parent(&self) -> Option<Self> {
        if self.is_root() {
            None
        } else {
            Some(Self {
                parts: self.parts[0..self.parts.len() - 1].to_vec(),
            })
        }
    }

    pub fn pop(&mut self) -> Option<String> {
        self.parts.pop()
    }

    /// Split a canonical path in two parts, returning the containing folder and
    /// an optional name, similar to what the `name()` method returns.
    ///
    /// If the canonical path cannot be split because it contains only one part,
    /// `None` is returned as the second part.
    pub fn split(&self) -> (Self, Option<&str>) {
        if self.is_root() {
            return (self.clone(), None);
        }

        (
            Self {
                parts: self.parts[0..self.parts.len() - 1].to_vec(),
            },
            Some(self.parts[self.parts.len() - 1].as_str()),
        )
    }

    /// Split a canonical path in two parts from the left, if possible.
    ///
    /// If the canonical path cannot be split because it contains only one part, `None` is returned as the second part.
    pub(crate) fn split_left(&self) -> Option<(&str, Option<Self>)> {
        if self.is_root() {
            return None;
        }

        Some(if self.parts.len() > 1 {
            (
                &self.parts[0],
                Some(Self {
                    parts: self.parts[1..].to_vec(),
                }),
            )
        } else {
            (&self.parts[0], None)
        })
    }

    /// Returns the name of the file or directory designated by the canonical path.
    ///
    /// If the path indicates the root, `None` is returned.
    pub fn name(&self) -> Option<&str> {
        self.parts.last().map(|s| &**s)
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.matches(other) || other.matches(self)
    }

    pub fn matches(&self, other: &Self) -> bool {
        // If the other path is longer than this one, it cannot match.
        if self.parts.len() > other.parts.len() {
            return false;
        }

        for (i, part) in self.parts.iter().enumerate() {
            if part != &other.parts[i] {
                return false;
            }
        }

        true
    }
}

impl Display for CanonicalPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "/{}", self.parts.join("/"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cp(s: &str) -> CanonicalPath {
        CanonicalPath::new(s).unwrap()
    }

    #[test]
    fn test_canonical_path_new() {
        assert!(CanonicalPath::new("/").is_ok());
        assert!(CanonicalPath::new("/a").is_ok());
        assert!(CanonicalPath::new("/a/b").is_ok());
        assert!(CanonicalPath::new("").is_err());
        assert!(CanonicalPath::new("a/b").is_err());
        assert!(CanonicalPath::new("/a/").is_err());
        assert!(CanonicalPath::new("/a//b").is_err());
    }

    #[test]
    fn test_canonical_path_comparison() {
        assert_eq!(cp("/a"), cp("/a"));
        assert_ne!(cp("/a"), cp("/b"));
    }

    #[test]
    fn test_canonical_path_ordering() {
        assert!(cp("/a") <= cp("/a"));
        assert!(!(cp("/a") < cp("/a")));

        assert!(cp("/a/b") > cp("/a"));
        assert!(cp("/a/b") < cp("/b"));
    }

    #[test]
    fn test_canonical_path_to_path_buf() {
        assert_eq!(
            cp("/a").to_path_buf(Path::new("/foo/bar")),
            Path::new("/foo/bar/a")
        );
    }

    #[test]
    fn test_canonical_path_is_root() {
        assert!(!cp("/a").is_root());
        assert!(cp("/").is_root());
    }

    #[test]
    fn test_canonical_path_name() {
        assert_eq!(cp("/a").name(), Some("a"));
        assert_eq!(cp("/a/b/c/d").name(), Some("d"));
        assert_eq!(cp("/").name(), None);
    }

    #[test]
    fn test_canonical_path_join() {
        assert_eq!(cp("/a/b/c/d").join(&cp("/e")), cp("/a/b/c/d/e"));
        assert_eq!(cp("/").join(&cp("/a")), cp("/a"));
        assert_eq!(cp("/").join(&cp("/")), cp("/"));
        assert_eq!(cp("/a").join(&cp("/")), cp("/a"));
    }

    #[test]
    fn test_canonical_path_prepend() {
        assert_eq!(cp("/a/b/c/d").prepend("e"), cp("/e/a/b/c/d"));
        assert_eq!(cp("/").prepend("a"), cp("/a"));
        assert_eq!(cp("/").prepend(""), cp("/"));
        assert_eq!(cp("/a").prepend(""), cp("/a"));
    }

    #[test]
    fn test_canonical_path_append() {
        assert_eq!(cp("/a/b/c/d").append("e"), cp("/a/b/c/d/e"));
        assert_eq!(cp("/").append("a"), cp("/a"));
        assert_eq!(cp("/").append(""), cp("/"));
        assert_eq!(cp("/a").append(""), cp("/a"));
    }

    #[test]
    fn test_canonical_path_split() {
        assert_eq!(cp("/a/b/c/d").split(), (cp("/a/b/c"), Some("d")));
        assert_eq!(cp("/a").split(), (cp("/"), Some("a")));
        assert_eq!(cp("/").split(), (cp("/"), None));
    }

    #[test]
    fn test_canonical_path_split_left() {
        assert_eq!(cp("/a/b/c/d").split_left(), Some(("a", Some(cp("/b/c/d")))));
        assert_eq!(cp("/a").split_left(), Some(("a", None)));
        assert_eq!(cp("/").split_left(), None);
    }

    #[test]
    fn test_canonical_path_intersects() {
        assert!(cp("/a").intersects(&cp("/a")));
        assert!(cp("/a").intersects(&cp("/a/b")));
        assert!(cp("/a/b").intersects(&cp("/a")));
        assert!(cp("/").intersects(&cp("/a")));
        assert!(cp("/a").intersects(&cp("/")));
        assert!(!cp("/x").intersects(&cp("/a/b")));
        assert!(!cp("/a/b").intersects(&cp("/x")));
    }
    #[test]
    fn test_canonical_path_matches() {
        assert!(cp("/a").matches(&cp("/a")));
        assert!(cp("/a").matches(&cp("/a/b")));
        assert!(cp("/").matches(&cp("/a")));
        assert!(!cp("/x").matches(&cp("/a/b")));
        assert!(!cp("/a/b").matches(&cp("/a")));
    }
}
