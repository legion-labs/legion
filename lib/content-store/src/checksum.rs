use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use siphasher::sip128;

/// Represents the checksum of a content file, as an unsigned 128-bit value.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Checksum(u128);

impl Checksum {
    /// Return the memory representation of this integer as a byte array in big-endian (network) byte order.
    pub const fn to_be_bytes(self) -> [u8; 16] {
        self.0.to_be_bytes()
    }
}

impl From<u128> for Checksum {
    fn from(value: u128) -> Self {
        Self(value)
    }
}

impl From<sip128::Hash128> for Checksum {
    fn from(value: sip128::Hash128) -> Self {
        value.as_u128().into()
    }
}

impl fmt::Display for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:032x}", self.0))
    }
}

impl FromStr for Checksum {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = u128::from_str_radix(s, 16)?;
        Ok(Self(value))
    }
}

impl Serialize for Checksum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let bytes = self.0.to_be_bytes();
            let hex = hex::encode(bytes);
            serializer.serialize_str(&hex)
        } else {
            serializer.serialize_u128(self.0)
        }
    }
}

impl<'de> Deserialize<'de> for Checksum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let value = {
            if deserializer.is_human_readable() {
                let hex = String::deserialize(deserializer)?;
                let digits = hex::decode(hex).map_err(D::Error::custom)?;
                u128::from_be_bytes(digits.try_into().unwrap())
            } else {
                u128::deserialize(deserializer)?
            }
        };
        Ok(value.into())
    }
}
