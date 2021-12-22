use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::Result;
use reqwest::Url;

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
        let validation = RefCell::new(None);

        let violation_cb = |violation| {
            *validation.borrow_mut() = Some(violation);
        };

        let options = Url::options().syntax_violation_callback(Some(&violation_cb));

        let result = options.parse(s);
        let validation = *validation.borrow();

        match (result, validation) {
            (_, Some(url::SyntaxViolation::ExpectedFileDoubleSlash)) => {
                Err(anyhow::anyhow!("expected file://"))
            }
            (Ok(url), _) => match url.scheme() {
                "file" => Ok(Self::Local(url.to_file_path().unwrap())),
                "mysql" => Ok(Self::MySQL(url)),
                "lsc" => Ok(Self::Lsc(url)),
                _ => Ok(Self::Local(s.into())),
            },
            (Err(_), Some(validation)) => Err(anyhow::anyhow!("{}", validation)),
            (Err(_), None) => Ok(Self::Local(s.into())),
        }
    }
}

impl RepositoryUrl {
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
}
