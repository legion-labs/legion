use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    any::{Any, TypeId},
    convert::TryInto,
    fmt,
    hash::{Hash, Hasher},
    io,
    str::FromStr,
};

use crate::ResourceType;

/// Checksum of a resource.
#[derive(Copy, Clone, Debug, Eq)]
pub struct ResourceChecksum(i128);

impl ResourceChecksum {
    /// Retrieve value of checksum as a signed 128 bit integer.
    pub fn get(&self) -> i128 {
        self.0
    }
}

impl PartialEq for ResourceChecksum {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Hash for ResourceChecksum {
    fn hash<H: Hasher>(&self, mut state: &mut H) {
        self.0.hash(&mut state);
    }
}

impl From<i128> for ResourceChecksum {
    fn from(value: i128) -> Self {
        Self(value)
    }
}

impl From<ResourceChecksum> for i128 {
    fn from(value: ResourceChecksum) -> Self {
        value.0
    }
}

impl fmt::Display for ResourceChecksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:032x}", self.0))
    }
}

impl FromStr for ResourceChecksum {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = i128::from_str_radix(s, 16)?;
        Ok(Self(value))
    }
}

impl Serialize for ResourceChecksum {
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

impl<'de> Deserialize<'de> for ResourceChecksum {
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
pub trait Resource: Any + Send + Sync {}

/// Trait describing assets type and its loader
pub trait AssetDescriptor {
    /// Name of the asset type.
    const TYPENAME: &'static str;
    /// Type of the asset.
    const TYPE: ResourceType = ResourceType::new(Self::TYPENAME.as_bytes());
    /// Loader of the asset.
    type Loader: AssetLoader + Send + Default + 'static;
}

/// Note: Based on impl of dyn Any
impl dyn Resource {
    /// Returns `true` if the boxed type is the same as `T`.
    /// (See [`std::any::Any::is`](https://doc.rust-lang.org/std/any/trait.Any.html#method.is))
    #[inline]
    pub fn is<T: Resource>(&self) -> bool {
        TypeId::of::<T>() == self.type_id()
    }

    /// Returns some reference to the boxed value if it is of type `T`, or
    /// `None` if it isn't.
    /// (See [`std::any::Any::downcast_ref`](https://doc.rust-lang.org/std/any/trait.Any.html#method.downcast_ref))
    #[inline]
    pub fn downcast_ref<T: Resource>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe { Some(&*((self as *const dyn Resource).cast::<T>())) }
        } else {
            None
        }
    }

    /// Returns some mutable reference to the boxed value if it is of type `T`, or
    /// `None` if it isn't.
    /// (See [`std::any::Any::downcast_mut`](https://doc.rust-lang.org/std/any/trait.Any.html#method.downcast_mut))
    #[inline]
    pub fn downcast_mut<T: Resource>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            unsafe { Some(&mut *((self as *mut dyn Resource).cast::<T>())) }
        } else {
            None
        }
    }
}

/// Note: Based on impl of dyn Any
impl dyn Resource + Send + Sync {
    /// Returns `true` if the boxed type is the same as `T`.
    /// (See [`std::any::Any::is`](https://doc.rust-lang.org/std/any/trait.Any.html#method.is))
    #[inline]
    pub fn is<T: Resource>(&self) -> bool {
        TypeId::of::<T>() == self.type_id()
    }

    /// Returns some reference to the boxed value if it is of type `T`, or
    /// `None` if it isn't.
    /// (See [`std::any::Any::downcast_ref`](https://doc.rust-lang.org/std/any/trait.Any.html#method.downcast_ref))
    #[inline]
    pub fn downcast_ref<T: Resource>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe { Some(&*((self as *const dyn Resource).cast::<T>())) }
        } else {
            None
        }
    }

    /// Returns some mutable reference to the boxed value if it is of type `T`, or
    /// `None` if it isn't.
    /// (See [`std::any::Any::downcast_mut`](https://doc.rust-lang.org/std/any/trait.Any.html#method.downcast_mut))
    #[inline]
    pub fn downcast_mut<T: Resource>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            unsafe { Some(&mut *((self as *mut dyn Resource).cast::<T>())) }
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
        kind: ResourceType,
        reader: &mut dyn io::Read,
    ) -> Result<Box<dyn Resource + Send + Sync>, io::Error>;

    /// Asset initialization executed after the asset and all its dependencies
    /// have been loaded.
    fn load_init(&mut self, asset: &mut (dyn Resource + Send + Sync));
}
