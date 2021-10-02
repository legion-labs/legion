use std::{
    convert::{TryFrom, TryInto},
    fmt,
    str::FromStr,
};

use rand::Rng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Type identifier of resource or asset.
///
/// It is currently generated randomly by hashing a byte array. It uses [`Self::num_bits`] number of bits.
/// In the future, it can be optimized to use less bits to leave more bits for the asset/resource identifier.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Debug)]
pub struct ResourceType(u32);

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:08x}", self.0))
    }
}

impl ResourceType {
    const CRC32_ALGO: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_CKSUM);

    const fn crc32(v: &[u8]) -> u32 {
        Self::CRC32_ALGO.checksum(v)
    }

    /// Number of bits used to represent `ContentType`.
    pub const fn num_bits() -> u32 {
        u32::BITS
    }

    /// Creates a new [`Self::num_bits`]-bit type id from series of bytes.
    ///
    /// It is recommended to use this method to define a public constant
    /// which can be used to identify a resource or asset.
    pub const fn new(v: &[u8]) -> Self {
        // TODO: A std::num::NonZeroU32 would be more suitable as an internal representation
        // however a value of 0 is as likely as any other value returned by `crc32`
        // and const-fn-friendly panic is not available yet.
        // See https://github.com/rust-lang/rfcs/pull/2345.
        let v = Self::crc32(v);
        Self(v)
    }

    /// Creates a [`Self::num_bits`]-bit type id from a non-zero integer.
    pub fn from_raw(v: u32) -> Self {
        Self(v)
    }

    /// Replaces [`Self::num_bits`] most significant bits of id with the content type id.
    pub fn stamp(&self, id: u128) -> u128 {
        let value_bits = u128::BITS - Self::num_bits();
        ((self.0 as u128) << value_bits) | (id & ((1 << value_bits) - 1))
    }
}

/// Id of a runtime asset or source or derived resource.
#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Debug, Hash)]
pub struct ResourceId(std::num::NonZeroU128);

impl ResourceId {
    /// Creates a new id of a given type.
    pub fn new(kind: ResourceType, id: u64) -> Self {
        let internal = kind.stamp(id as u128);
        Self(std::num::NonZeroU128::new(internal).unwrap())
    }

    /// Creates a new random id.
    pub fn new_random_id(kind: ResourceType) -> Self {
        let rand_id: u64 = rand::thread_rng().gen();
        Self::new(kind, rand_id)
    }

    /// Returns the type of `ContentId`.
    pub fn ty(&self) -> ResourceType {
        let type_id = (u128::from(self.0) >> (u128::BITS - ResourceType::num_bits())) as u32;
        ResourceType::from_raw(type_id)
    }
}

impl fmt::LowerHex for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:032x}", self.0))
    }
}

impl FromStr for ResourceId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = u128::from_str_radix(s, 16)?;
        if id == 0 {
            Err("Z".parse::<i32>().expect_err("ParseIntError"))
        } else {
            // SAFETY: id is not zero in this else clause.
            let id = unsafe { std::num::NonZeroU128::new_unchecked(id) };
            Ok(Self(id))
        }
    }
}

impl TryFrom<u128> for ResourceId {
    type Error = ();

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        let id = std::num::NonZeroU128::new(value).ok_or(())?;
        Ok(Self(id))
    }
}

impl Serialize for ResourceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let bytes = self.0.get().to_be_bytes();
            let hex = hex::encode(bytes);
            serializer.serialize_str(&hex)
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
        use serde::de::Error;

        let id = {
            if deserializer.is_human_readable() {
                let hex = String::deserialize(deserializer)?;
                let digits = hex::decode(hex).map_err(D::Error::custom)?;
                u128::from_be_bytes(digits.try_into().unwrap())
            } else {
                u128::deserialize(deserializer)?
            }
        };
        match std::num::NonZeroU128::new(id) {
            Some(id) => Ok(Self(id)),
            None => Err(D::Error::custom("invalid id")),
        }
    }
}
