use core::fmt;
use legion_content_store::ContentType;
use serde::{Deserialize, Serialize};
use std::{
    any::{Any, TypeId},
    fmt::LowerHex,
    hash::Hash,
    io,
};

/// A unique id of a runtime asset.
///
/// This 64 bit id encodes the following information:
/// - asset unique id - 32 bits
/// - [`AssetType`] - 32 bits
#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct AssetId {
    id: std::num::NonZeroU64,
}

impl AssetId {
    /// Creates an asset id of a given type.
    pub fn new(kind: AssetType, id: u32) -> Self {
        let internal = kind.stamp(id as u64);
        Self {
            id: std::num::NonZeroU64::new(internal).unwrap(),
        }
    }

    /// Creates an asset id from a raw hash value.
    pub fn from_hash_id(id: u64) -> Option<Self> {
        std::num::NonZeroU64::new(id).map(|id| Self { id })
    }

    /// Returns the type of the asset.
    pub fn asset_type(&self) -> AssetType {
        let type_id = (u64::from(self.id) >> 32) as u32;
        AssetType::from_raw(type_id)
    }
}

impl LowerHex for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::LowerHex::fmt(&self.id, f)
    }
}

impl ToString for AssetId {
    fn to_string(&self) -> String {
        self.id.to_string()
    }
}

/// Type id of a runtime asset.
pub type AssetType = ContentType;

pub use legion_data_runtime_macros::Asset;

/// Types implementing `Asset` represent non-mutable runtime data.
pub trait Asset: Any + Send {}

/// Note: Based on impl of dyn Any
impl dyn Asset {
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

/// Note: Based on impl of dyn Any
impl dyn Asset + Send + Sync {
    /// Returns `true` if the boxed type is the same as `T`.
    /// (See [`std::any::Any::is`](https://doc.rust-lang.org/std/any/trait.Any.html#method.is))
    #[inline]
    pub fn is<T: Asset>(&self) -> bool {
        <dyn Asset>::is::<T>(self)
    }

    /// Returns some reference to the boxed value if it is of type `T`, or
    /// `None` if it isn't.
    /// (See [`std::any::Any::downcast_ref`](https://doc.rust-lang.org/std/any/trait.Any.html#method.downcast_ref))
    #[inline]
    pub fn downcast_ref<T: Asset>(&self) -> Option<&T> {
        <dyn Asset>::downcast_ref::<T>(self)
    }

    /// Returns some mutable reference to the boxed value if it is of type `T`, or
    /// `None` if it isn't.
    /// (See [`std::any::Any::downcast_mut`](https://doc.rust-lang.org/std/any/trait.Any.html#method.downcast_mut))
    #[inline]
    pub fn downcast_mut<T: Asset>(&mut self) -> Option<&mut T> {
        <dyn Asset>::downcast_mut::<T>(self)
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
