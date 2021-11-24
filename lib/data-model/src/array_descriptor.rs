use crate::{BaseDescriptor, TypeDefinition, TypeReflection};
use thiserror::Error;

/// Define the reflection of a Array typpe
pub struct ArrayDescriptor {
    /// Base Descriptor
    pub base_descriptor: BaseDescriptor,
    /// Type of the Vec Element
    pub inner_type: TypeDefinition,
    /// Function to return the array size
    pub len: unsafe fn(array: *const ()) -> usize,
    /// Function to return an element raw pointer
    pub get: unsafe fn(array: *const (), index: usize) -> anyhow::Result<*const ()>,
    /// Function to return an element mutable raw pointer
    pub get_mut: unsafe fn(array: *mut (), index: usize) -> anyhow::Result<*mut ()>,
}

#[derive(Error, Debug)]
/// `ArrayDescriptor` Error
pub enum ArrayDescriptorError {
    /// Error when accessing out of bounds index
    #[error("Invalid array index {0} on ArrayDescriptor: '{1}'")]
    InvalidArrayIndex(usize, &'static str),
}

/// Macro to implement array descriptor
#[macro_export]
macro_rules! implement_array_descriptor {
    ($type_id:ty) => {
        lazy_static::lazy_static! {
            static ref ARRAY_DESCRIPTOR: $crate::ArrayDescriptor = $crate::ArrayDescriptor {
                base_descriptor : $crate::create_base_descriptor!(Vec<$type_id>, concat!("Vec<",stringify!($type_id),">").into()),
                inner_type: <$type_id as $crate::TypeReflection>::get_type_def(),
                len: |array: *const ()| unsafe { (*(array as *const Vec<$type_id>)).len() },
                get: |array: *const (), index: usize| unsafe {
                    (*(array as *const Vec<$type_id>))
                        .get(index).ok_or($crate::ArrayDescriptorError::InvalidArrayIndex(index, concat!("Vec<",stringify!($type_id),">")).into())
                        .and_then(|value| Ok((value as *const $type_id).cast::<()>()))
                    },
                get_mut:|array: *mut (), index: usize| unsafe {
                    (*(array as *mut Vec<$type_id>))
                        .get_mut(index).ok_or($crate::ArrayDescriptorError::InvalidArrayIndex(index, concat!("Vec<",stringify!($type_id),">")).into())
                        .and_then(|value| Ok((value as *mut $type_id).cast::<()>()))
                },
            };
        }
    };
}

impl<T: TypeReflection> TypeReflection for Vec<T> {
    fn get_type(&self) -> TypeDefinition {
        Self::get_type_def()
    }

    fn get_type_def() -> TypeDefinition {
        T::get_array_def()
    }
}
