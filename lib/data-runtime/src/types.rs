use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    any::{Any, TypeId},
    convert::{TryFrom, TryInto},
    fmt,
    hash::{Hash, Hasher},
    io,
    str::FromStr,
};

use crate::{ContentId, ContentType};

/// A unique id of a runtime asset.
///
/// This 64 bit id encodes the following information:
/// - asset unique id - 32 bits
/// - [`AssetType`] - 32 bits
#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct AssetId(ContentId);

impl AssetId {
    /// Creates an asset id of a given type.
    pub fn new(kind: AssetType, id: u64) -> Self {
        Self(ContentId::new(kind.into(), id))
    }

    /// Returns the type of the asset.
    pub fn asset_type(&self) -> AssetType {
        AssetType(self.0.kind())
    }
}

impl fmt::LowerHex for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::Display for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl TryFrom<ContentId> for AssetId {
    type Error = ();

    fn try_from(value: ContentId) -> Result<Self, Self::Error> {
        if !value.kind().is_rt() {
            return Err(());
        }
        Ok(Self(value))
    }
}

/// Type id of a runtime asset.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Debug)]
pub struct AssetType(ContentType);

impl AssetType {
    /// Creates a new type id from a byte array.
    ///
    /// It is recommended to use this method to define a public constant
    /// which can be used to identify an asset type.
    pub const fn new(v: &[u8]) -> Self {
        Self(ContentType::new(v, true))
    }

    /// Returns underlying id (at compile-time).
    pub const fn content(&self) -> ContentType {
        self.0
    }
}

impl TryFrom<ContentType> for AssetType {
    type Error = ();

    fn try_from(value: ContentType) -> Result<Self, Self::Error> {
        match value.is_rt() {
            true => Ok(Self(value)),
            false => Err(()),
        }
    }
}

impl From<AssetType> for ContentType {
    fn from(value: AssetType) -> Self {
        value.0
    }
}

impl FromStr for AssetId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ContentId::from_str(s)?
            .try_into()
            .map_err(|_e| "Z".parse::<i32>().expect_err("ParseIntError"))
    }
}

/// Checksum of a runtime asset.
#[derive(Copy, Clone, Debug, Eq)]
pub struct AssetChecksum(i128);

impl AssetChecksum {
    /// Retrieve value of checksum as a signed 128 bit integer.
    pub fn get(&self) -> i128 {
        self.0
    }
}

impl PartialEq for AssetChecksum {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Hash for AssetChecksum {
    fn hash<H: Hasher>(&self, mut state: &mut H) {
        self.0.hash(&mut state);
    }
}

impl From<i128> for AssetChecksum {
    fn from(value: i128) -> Self {
        Self(value)
    }
}

impl From<AssetChecksum> for i128 {
    fn from(value: AssetChecksum) -> Self {
        value.0
    }
}

impl fmt::Display for AssetChecksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:032x}", self.0))
    }
}

impl FromStr for AssetChecksum {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = i128::from_str_radix(s, 16)?;
        Ok(Self(value))
    }
}

impl Serialize for AssetChecksum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let bytes = self.0.to_be_bytes();
            let hex = hex::encode(bytes);
            serializer.serialize_str(&hex)
        } else {
            serializer.serialize_i128(self.0)
        }
    }
}

impl<'de> Deserialize<'de> for AssetChecksum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let value = {
            if deserializer.is_human_readable() {
                let hex = String::deserialize(deserializer)?;
                let digits = hex::decode(hex).map_err(D::Error::custom)?;
                i128::from_be_bytes(digits.try_into().unwrap())
            } else {
                i128::deserialize(deserializer)?
            }
        };
        Ok(value.into())
    }
}

/// Types implementing `Asset` represent non-mutable runtime data.
pub trait Asset: Any + Send + Sync {}

/// Note: Based on impl of dyn Any
impl dyn Asset + Send + Sync {
    /// Returns `true` if the boxed type is the same as `T`.
    /// (See [`std::any::Any::is`](https://doc.rust-lang.org/std/any/trait.Any.html#method.is))
    #[inline]
    pub fn is<T: Asset>(&self) -> bool {
        TypeId::of::<T>() == self.type_id()
    }

    /// Returns some reference to the boxed value if it is of type `T`, or
    /// `None` if it isn't.
    /// (See [`std::any::Any::downcast_ref`](https://doc.rust-lang.org/std/any/trait.Any.html#method.downcast_ref))
    #[inline]
    pub fn downcast_ref<T: Asset>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe { Some(&*((self as *const dyn Asset).cast::<T>())) }
        } else {
            None
        }
    }

    /// Returns some mutable reference to the boxed value if it is of type `T`, or
    /// `None` if it isn't.
    /// (See [`std::any::Any::downcast_mut`](https://doc.rust-lang.org/std/any/trait.Any.html#method.downcast_mut))
    #[inline]
    pub fn downcast_mut<T: Asset>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            unsafe { Some(&mut *((self as *mut dyn Asset).cast::<T>())) }
        } else {
            None
        }
    }
}

/// An interface allowing to create and initialize assets.
pub trait AssetLoader {
    /// Asset loading interface.
    fn load(
        &mut self,
        kind: AssetType,
        reader: &mut dyn io::Read,
    ) -> Result<Box<dyn Asset + Send + Sync>, io::Error>;

    /// Asset initialization executed after the asset and all its dependencies
    /// have been loaded.
    fn load_init(&mut self, asset: &mut (dyn Asset + Send + Sync));
}
