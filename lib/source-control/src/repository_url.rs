use anyhow::Result;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{parse_url_or_path, UrlOrPath};

/// A repository URL.
///
/// This type represents all the possible types of repositories and their
/// designative URLs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RepositoryUrl {
    Local(PathBuf),
    MySQL(Url),
    Lsc(Url),
}

impl FromStr for RepositoryUrl {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match parse_url_or_path(s)? {
            UrlOrPath::Path(path) => Ok(Self::Local(path)),
            UrlOrPath::Url(url) => match url.scheme() {
                "mysql" => Ok(Self::MySQL(url)),
                "lsc" => Ok(Self::Lsc(url)),
                scheme => Err(anyhow::anyhow!(
                    "unsupported repository URL scheme: {}",
                    scheme
                )),
            },
        }
    }
}

impl RepositoryUrl {
    /// Create a repository URL from the current directory.
    pub fn from_current_dir() -> Self {
        Self::Local(".".into())
    }

    /// Make the repository URL absolute, possibly using the specified path if
    /// the URL is a local relative repository URL.
    ///
    /// In any other case, the URL is returned as is.
    pub fn make_absolute(self, base: impl AsRef<Path>) -> Self {
        if let Self::Local(r) = self {
            Self::Local(base.as_ref().join(r))
        } else {
            self
        }
    }
}

impl Display for RepositoryUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local(p) => write!(f, "{}", p.display()),
            Self::MySQL(u) | Self::Lsc(u) => write!(f, "{}", u),
        }
    }
}

impl Serialize for RepositoryUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for RepositoryUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_file() {
        #[cfg(not(windows))]
        assert_eq!(
            RepositoryUrl::from_str("file:///home/user/repo").unwrap(),
            RepositoryUrl::Local(PathBuf::from("/home/user/repo"))
        );
        #[cfg(windows)]
        assert_eq!(
            RepositoryUrl::from_str(r"file:///C:/Users/user/repo").unwrap(),
            RepositoryUrl::Local(PathBuf::from(r"C:/Users/user/repo"))
        );
        #[cfg(windows)]
        assert_eq!(
            RepositoryUrl::from_str(r"file:///C:\Users\user\repo").unwrap(),
            RepositoryUrl::Local(PathBuf::from(r"C:/Users/user/repo"))
        );
    }

    #[test]
    fn test_from_str_file_no_scheme() {
        #[cfg(not(windows))]
        assert_eq!(
            RepositoryUrl::from_str("/home/user/repo").unwrap(),
            RepositoryUrl::Local(PathBuf::from("/home/user/repo"))
        );
        #[cfg(windows)]
        assert_eq!(
            RepositoryUrl::from_str(r"C:/Users/user/repo").unwrap(),
            RepositoryUrl::Local(PathBuf::from(r"C:/Users/user/repo"))
        );
        #[cfg(windows)]
        assert_eq!(
            RepositoryUrl::from_str(r"C:\Users\user\repo").unwrap(),
            RepositoryUrl::Local(PathBuf::from(r"C:/Users/user/repo"))
        );
    }

    #[test]
    fn test_from_str_file_no_scheme_relative() {
        assert_eq!(
            RepositoryUrl::from_str("repo").unwrap(),
            RepositoryUrl::Local(PathBuf::from("repo"))
        );
    }

    #[test]
    fn test_from_str_mysql() {
        assert_eq!(
            RepositoryUrl::from_str("mysql://user:pass@localhost:3306/db").unwrap(),
            RepositoryUrl::MySQL(Url::parse("mysql://user:pass@localhost:3306/db").unwrap())
        );
    }

    #[test]
    fn test_from_str_lsc() {
        assert_eq!(
            RepositoryUrl::from_str("lsc://user:pass@localhost:3306/db").unwrap(),
            RepositoryUrl::Lsc(Url::parse("lsc://user:pass@localhost:3306/db").unwrap())
        );
    }

    #[test]
    #[should_panic]
    fn test_from_str_unsupported() {
        RepositoryUrl::from_str("file:").unwrap();
    }

    #[test]
    fn test_make_absolute() {
        let url: RepositoryUrl = "repo".parse().unwrap();
        let url = url.make_absolute("/home/user");

        assert_eq!(url, RepositoryUrl::Local(PathBuf::from("/home/user/repo")));
    }

    #[test]
    fn test_display() {
        assert_eq!(
            RepositoryUrl::Local("my/path".into()).to_string(),
            "my/path"
        );
        assert_eq!(
            RepositoryUrl::MySQL("mysql://user:pass@localhost:3306/db".try_into().unwrap())
                .to_string(),
            "mysql://user:pass@localhost:3306/db"
        );
        assert_eq!(
            RepositoryUrl::Lsc("lsc://user:pass@localhost:3306/db".try_into().unwrap()).to_string(),
            "lsc://user:pass@localhost:3306/db"
        );
    }
}
