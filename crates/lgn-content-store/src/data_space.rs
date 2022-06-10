use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct DataSpace(String);

impl Display for DataSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for DataSpace {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            Self::PERSISTENT => Ok(Self::persistent()),
            Self::VOLATILE => Ok(Self::volatile()),
            _ => Err(Error::InvalidDataSpace(s.to_string())),
        }
    }
}

impl From<DataSpace> for crate::api::content_store::DataSpace {
    fn from(s: DataSpace) -> Self {
        Self(s.0)
    }
}

impl TryFrom<crate::api::content_store::DataSpace> for DataSpace {
    type Error = Error;

    fn try_from(s: crate::api::content_store::DataSpace) -> Result<Self, Self::Error> {
        Self::from_str(&s.0)
    }
}

impl<'de> Deserialize<'de> for DataSpace {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl DataSpace {
    const PERSISTENT: &'static str = "persistent";
    const VOLATILE: &'static str = "volatile";

    pub fn persistent() -> Self {
        Self(Self::PERSISTENT.to_string())
    }

    pub fn volatile() -> Self {
        Self(Self::VOLATILE.to_string())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::DataSpace;

    #[test]
    fn test_deserialize_data_space() {
        let data_space = serde_json::from_value(json!("persistent")).unwrap();

        assert_eq!(DataSpace::persistent(), data_space);

        let data_space = serde_json::from_value(json!("volatile")).unwrap();

        assert_eq!(DataSpace::volatile(), data_space);

        assert!(serde_json::from_value::<DataSpace>(json!("invalid")).is_err());
    }
}
