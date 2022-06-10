use std::any::{Any, TypeId};

use bincode::{DefaultOptions, Options};
use lgn_data_model::TypeReflection;

use crate::{AssetRegistryError, AssetRegistryReader, ResourceType, ResourceTypeEntry};

/// Trait describing resource type name.
pub trait ResourceDescriptor {
    /// Name of the asset type.
    const TYPENAME: &'static str;
    /// Type of the asset.
    const TYPE: ResourceType = ResourceType::new(Self::TYPENAME.as_bytes());
}

/// Create a Resource from a binary stream
/// # Errors
/// Return `AssetRegistryError` on failure
pub async fn from_binary_reader<'de, T: Resource + Default + serde::Deserialize<'de>>(
    reader: &mut AssetRegistryReader,
) -> Result<Box<T>, AssetRegistryError> {
    use tokio::io::AsyncReadExt;
    let mut buffer = vec![];
    reader.read_to_end(&mut buffer).await?;
    let cursor = std::io::Cursor::new(buffer);

    let mut deserializer = bincode::de::Deserializer::with_reader(
        cursor,
        DefaultOptions::new()
            .allow_trailing_bytes()
            .with_fixint_encoding(),
    );

    let mut new_resource = Box::new(T::default());
    serde::Deserialize::deserialize_in_place(&mut deserializer, &mut new_resource)
        .map_err(|err| AssetRegistryError::SerializationFailed("", err.to_string()))?;

    Ok(new_resource)
}

/// Write a Resource to a binary stream
/// # Errors
/// Return `AssetRegistryError` on failure
pub fn to_binary_writer(
    resource: &dyn Resource,
    writer: &mut dyn std::io::Write,
) -> Result<(), AssetRegistryError> {
    let mut bincode_ser = bincode::Serializer::new(
        writer,
        DefaultOptions::new()
            .allow_trailing_bytes()
            .with_fixint_encoding(),
    );
    let mut serializer = <dyn erased_serde::Serializer>::erase(&mut bincode_ser);
    lgn_data_model::utils::serialize_property_by_name(resource.as_reflect(), "", &mut serializer)?;

    Ok(())
}

/// Trait describing a resource
#[async_trait::async_trait]
pub trait Resource: TypeReflection + Any + Send + Sync {
    /// Return the `Resource` as a reflected type
    fn as_reflect(&self) -> &dyn TypeReflection;

    /// Return the `Resource` as a reflected type
    fn as_reflect_mut(&mut self) -> &mut dyn TypeReflection;

    /// Return a shallow clone of the Resource
    fn clone_dyn(&self) -> Box<dyn Resource>;

    /// Return the ResourceType of a Resource
    fn get_resource_type(&self) -> ResourceType;

    /// Registry the ResourceType
    fn register_resource_type()
    where
        Self: Sized + ResourceDescriptor + Default,
    {
        ResourceType::register_type(
            Self::TYPE,
            ResourceTypeEntry {
                name: Self::TYPENAME,
                new_instance: || Box::new(Self::default()),
            },
        );
    }
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
    #[inline]
    pub fn downcast_ref<T: Resource>(&self) -> Option<&T> {
        if self.is::<T>() {
            #[allow(unsafe_code)]
            unsafe {
                Some(&*((self as *const dyn Resource).cast::<T>()))
            }
        } else {
            None
        }
    }

    /// Returns some reference to the boxed value if it is of type `T`, or
    #[inline]
    pub fn downcast_mut<T: Resource>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            #[allow(unsafe_code)]
            unsafe {
                Some(&mut *((self as *mut dyn Resource).cast::<T>()))
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsStr, path::PathBuf, str::FromStr};

    use crate::ResourceId;

    #[test]
    fn resource_path() {
        let text = "986a4ba3-d1d0-43ca-9051-56d26ad421ad";
        let id = ResourceId::from_str(text).expect("valid uuid");
        let path: PathBuf = id.resource_path();

        let mut iter = path.iter();
        assert_eq!(iter.next(), Some(OsStr::new("98")));
        assert_eq!(iter.next(), Some(OsStr::new("6a")));
        assert_eq!(iter.next(), Some(OsStr::new("4b")));
        assert_eq!(
            iter.next(),
            Some(OsStr::new("986a4ba3-d1d0-43ca-9051-56d26ad421ad"))
        );
        assert_eq!(iter.next(), None);
    }
}
