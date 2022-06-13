use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// Represents the name of a branch.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct BranchName(String);

impl BranchName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for BranchName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for BranchName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.chars().all(Self::is_valid_char) {
            Ok(Self(s.to_owned()))
        } else {
            Err(Error::InvalidBranchName {
                branch_name: s.to_owned(),
                reason: "branch name characters must be alphanumeric".to_string(),
            })
        }
    }
}

impl From<BranchName> for crate::api::source_control::BranchName {
    fn from(name: BranchName) -> Self {
        Self(name.0)
    }
}

impl TryFrom<crate::api::source_control::BranchName> for BranchName {
    type Error = Error;
    fn try_from(name: crate::api::source_control::BranchName) -> Result<Self> {
        Self::from_str(&name.0)
    }
}

impl BranchName {
    fn is_valid_char(c: char) -> bool {
        c.is_alphanumeric() || c == '-' || c == '_' || c == '.'
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_name_from_str() {
        // These are valid.
        "test".parse::<BranchName>().unwrap();
        "123".parse::<BranchName>().unwrap();
        "123test".parse::<BranchName>().unwrap();
        "test123".parse::<BranchName>().unwrap();
        "also_valid".parse::<BranchName>().unwrap();
        "also-valid".parse::<BranchName>().unwrap();
        "also.valid".parse::<BranchName>().unwrap();

        // These are invalid.
        "in valid".parse::<BranchName>().unwrap_err();
        "in♥alid".parse::<BranchName>().unwrap_err();
    }
}
