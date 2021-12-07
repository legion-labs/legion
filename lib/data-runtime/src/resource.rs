use std::convert::TryFrom;
use std::{fmt, hash::Hash, str::FromStr};

use lgn_utils::DefaultHash;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;
use xxhash_rust::const_xxh3::xxh3_64 as const_xxh3;

/// Type identifier of resource or asset.
///
/// It is currently generated by hashing the name of a type, into a stable 32-bits value.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceType(std::num::NonZeroU32);

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:08x}", self.0))
    }
}
impl fmt::Debug for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ResourceType")
            .field(&format_args!("{:#08x}", self.0))
            .finish()
    }
}

impl ResourceType {
    /// Creates a new type id from series of bytes.
    ///
    /// It is recommended to use this method to define a public constant
    /// which can be used to identify a resource or asset.
    pub const fn new(v: &[u8]) -> Self {
        let v = const_xxh3(v) as u32;
        //Self(unsafe { std::num::NonZeroU32::new_unchecked(v) }) // unwrap() doesn't work with const functions ATM
        Self::from_raw(v)
    }

    /// Creates a type id from a non-zero integer.
    pub const fn from_raw(v: u32) -> Self {
        let v = match std::num::NonZeroU32::new(v) {
            Some(v) => v,
            None => panic!(),
        };
        Self(v)
    }
}

impl Serialize for ResourceType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let bytes = self.0.get().to_be_bytes();
            let hex = hex::encode(bytes);
            serializer.serialize_str(&hex)
        } else {
            serializer.serialize_u32(self.0.get())
        }
    }
}

impl<'de> Deserialize<'de> for ResourceType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let v = {
            if deserializer.is_human_readable() {
                let hex = String::deserialize(deserializer)?;
                let digits = hex::decode(hex).map_err(D::Error::custom)?;
                u32::from_be_bytes(digits.try_into().unwrap())
            } else {
                u32::deserialize(deserializer)?
            }
        };
        Ok(Self::from_raw(v))
    }
}

impl FromStr for ResourceType {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = u32::from_str_radix(s, 16)?;
        if v == 0 {
            Err("Z".parse::<i32>().expect_err("ParseIntError"))
        } else {
            Ok(Self::from_raw(v))
        }
    }
}

/// Id of a runtime asset or source or derived resource.
///
/// We currently use fully random 128-bit UUIDs, to ensure uniqueness without requiring a central authority.
/// This allows creation of two `ResourceId` on two separate machines and guarantee that we won't have any collision when submitting those Resources on the source control.
#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Debug, Hash)]
pub struct ResourceId(std::num::NonZeroU128);

impl ResourceId {
    /// Creates a new random id.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(std::num::NonZeroU128::new(Uuid::new_v4().as_u128()).unwrap())
    }
    /// Creates an explicit id, assuming that it is a runtime counter, not for serialization. The UUID 'version' is a non-standard value of 15.
    pub fn new_explicit(id: u64) -> Self {
        Self(
            std::num::NonZeroU128::new(
                uuid::Builder::from_u128(u128::from(id))
                    .set_version(unsafe { std::mem::transmute(0xF_u8) })
                    .build()
                    .as_u128(),
            )
            .unwrap(),
        )
    }

    /// Initialize from an existing, serialized, source.
    pub fn from_raw(id: u128) -> Self {
        Self(std::num::NonZeroU128::new(id).unwrap())
    }

    /// Initialize by hashing the contents of an object. We set 'Sha1' as UUID version even if our hash isn't really SHA-1.
    pub fn from_obj<T: Hash>(obj: &T) -> Self {
        let id = (*obj).default_hash_128();
        Self(
            std::num::NonZeroU128::new(
                uuid::Builder::from_u128(id)
                    .set_version(uuid::Version::Sha1)
                    .build()
                    .as_u128(),
            )
            .unwrap(),
        )
    }
}

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", Uuid::from_u128(self.0.get())))
    }
}

impl FromStr for ResourceId {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = Uuid::from_str(s)?;
        Ok(Self::from_raw(id.as_u128()))
    }
}

impl TryFrom<u128> for ResourceId {
    type Error = ();

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        Ok(Self::from_raw(value))
    }
}

impl Serialize for ResourceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let id = Uuid::from_u128(self.0.get()).to_string();
            serializer.serialize_str(&id)
        } else {
            serializer.serialize_u128(self.0.get())
        }
    }
}

impl<'de> Deserialize<'de> for ResourceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = {
            if deserializer.is_human_readable() {
                let id = String::deserialize(deserializer)?;
                Uuid::from_str(&id).unwrap().as_u128()
            } else {
                u128::deserialize(deserializer)?
            }
        };
        Ok(Self::from_raw(id))
    }
}

/// FIXME: This should only be a temporary struct, we should be using the `ResourceId` directly.
#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct ResourceTypeAndId(pub ResourceType, pub ResourceId);

impl FromStr for ResourceTypeAndId {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pair: Vec<&str> = s
            .trim_matches(|p| p == '(' || p == ')')
            .split(',')
            .collect();
        let t = pair[0].parse::<ResourceType>()?;
        let id = pair[1].parse::<ResourceId>()?;
        Ok(Self(t, id))
    }
}

impl fmt::Display for ResourceTypeAndId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("({},{})", self.0, self.1))
    }
}

/// Returns a string from a formatted `ResourceTypeAndId` tuple.
/*pub fn to_string(v: ResourceTypeAndId) -> String {
    format!("({},{})", v.0, v.1)
}*/

/// Trait describing resource type name.
pub trait Resource {
    /// Name of the asset type.
    const TYPENAME: &'static str;
    /// Type of the asset.
    const TYPE: ResourceType = ResourceType::new(Self::TYPENAME.as_bytes());
}
