use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::Error;

/// Represents the name of a repository.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct RepositoryName(String);

impl RepositoryName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for RepositoryName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for RepositoryName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.chars().all(RepositoryName::is_valid_char) {
            Ok(Self(s.to_owned()))
        } else {
            Err(Error::InvalidRepositoryName {
                repository_name: s.to_owned(),
                reason: "repository name characters must be alphanumeric".to_string(),
            })
        }
    }
}

impl RepositoryName {
    fn is_valid_char(c: char) -> bool {
        c.is_alphanumeric() || c == '-' || c == '_' || c == '.'
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_name_from_str() {
        // These are valid.
        "test".parse::<RepositoryName>().unwrap();
        "123".parse::<RepositoryName>().unwrap();
        "123test".parse::<RepositoryName>().unwrap();
        "test123".parse::<RepositoryName>().unwrap();
        "also_valid".parse::<RepositoryName>().unwrap();
        "also-valid".parse::<RepositoryName>().unwrap();
        "also.valid".parse::<RepositoryName>().unwrap();

        // These are invalid.
        "in valid".parse::<RepositoryName>().unwrap_err();
        "in♥alid".parse::<RepositoryName>().unwrap_err();
    }
}
