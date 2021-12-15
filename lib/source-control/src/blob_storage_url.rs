use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};
use url::Url;

use crate::{parse_url_or_path, UrlOrPath};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BlobStorageUrl {
    Local(PathBuf),
    AwsS3(Url),
}

impl BlobStorageUrl {
    /// Make the blob storage URL absolute, possibly using the specified path if
    /// the URL is a local relative blob storage URL.
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

impl FromStr for BlobStorageUrl {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match parse_url_or_path(s)? {
            UrlOrPath::Path(path) => Ok(Self::Local(path)),
            UrlOrPath::Url(url) => match url.scheme() {
                "s3" => Ok(Self::AwsS3(url)),
                scheme => Err(anyhow::anyhow!(
                    "unsupported repository URL scheme: {}",
                    scheme
                )),
            },
        }
    }
}

impl Display for BlobStorageUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local(p) => write!(f, "{}", p.display()),
            Self::AwsS3(u) => write!(f, "{}", u),
        }
    }
}

impl Serialize for BlobStorageUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for BlobStorageUrl {
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
            BlobStorageUrl::from_str("file:///home/user/repo").unwrap(),
            BlobStorageUrl::Local(PathBuf::from("/home/user/repo"))
        );
        #[cfg(windows)]
        assert_eq!(
            BlobStorageUrl::from_str(r"file:///C:/Users/user/repo").unwrap(),
            BlobStorageUrl::Local(PathBuf::from(r"C:/Users/user/repo"))
        );
        #[cfg(windows)]
        assert_eq!(
            BlobStorageUrl::from_str(r"file:///C:\Users\user\repo").unwrap(),
            BlobStorageUrl::Local(PathBuf::from(r"C:/Users/user/repo"))
        );
    }

    #[test]
    fn test_from_str_file_no_scheme() {
        #[cfg(not(windows))]
        assert_eq!(
            BlobStorageUrl::from_str("/home/user/repo").unwrap(),
            BlobStorageUrl::Local(PathBuf::from("/home/user/repo"))
        );
        #[cfg(windows)]
        assert_eq!(
            BlobStorageUrl::from_str(r"C:/Users/user/repo").unwrap(),
            BlobStorageUrl::Local(PathBuf::from(r"C:/Users/user/repo"))
        );
        #[cfg(windows)]
        assert_eq!(
            BlobStorageUrl::from_str(r"C:\Users\user\repo").unwrap(),
            BlobStorageUrl::Local(PathBuf::from(r"C:/Users/user/repo"))
        );
    }

    #[test]
    fn test_from_str_file_no_scheme_relative() {
        assert_eq!(
            BlobStorageUrl::from_str("repo").unwrap(),
            BlobStorageUrl::Local(PathBuf::from("repo"))
        );
    }

    #[test]
    fn test_from_str_aws_s3() {
        assert_eq!(
            BlobStorageUrl::from_str("s3://bucket/path").unwrap(),
            BlobStorageUrl::AwsS3(Url::parse("s3://bucket/path").unwrap())
        );
    }

    #[test]
    #[should_panic]
    fn test_from_str_unsupported() {
        BlobStorageUrl::from_str("file:").unwrap();
    }

    #[test]
    fn test_make_absolute() {
        let url: BlobStorageUrl = "repo".parse().unwrap();
        let url = url.make_absolute("/home/user");

        assert_eq!(url, BlobStorageUrl::Local(PathBuf::from("/home/user/repo")));
    }

    #[test]
    fn test_display() {
        assert_eq!(
            BlobStorageUrl::Local("my/path".into()).to_string(),
            "my/path"
        );
        assert_eq!(
            BlobStorageUrl::AwsS3("s3://bucket/path".try_into().unwrap()).to_string(),
            "s3://bucket/path"
        );
    }
}
