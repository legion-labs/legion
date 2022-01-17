use crate::{BaseDescriptor, TypeDefinition, TypeReflection};

/// Define the reflection of an Option<> type
pub struct OptionDescriptor {
    /// Base Descriptor of the Option Type
    pub base_descriptor: BaseDescriptor,
    /// Type of the inner type
    pub inner_type: TypeDefinition,
    /// Function to access inner_type
    pub get_inner: unsafe fn(option: *const ()) -> Option<*const ()>,
    /// Function to access the inner_type mut raw ptr
    pub get_inner_mut: unsafe fn(option: *mut ()) -> Option<*mut ()>,
}

/// Macro to implement an `OptionDescriptor` for a type
#[macro_export]
macro_rules! implement_option_descriptor {
    ($type_id:ty) => {
        lazy_static::lazy_static! {
            static ref OPTION_DESCRIPTOR: $crate::OptionDescriptor = $crate::OptionDescriptor {
                base_descriptor : $crate::create_base_descriptor!(Option<$type_id>, concat!("Option<",stringify!($type_id),">").into()),
                inner_type: <$type_id as $crate::TypeReflection>::get_type_def(),
                get_inner:  |option: *const ()| unsafe {
                    (*(option as *const Option<$type_id>))
                        .as_ref()
                        .map(|val| (val as *const $type_id).cast::<()>())
                },
                get_inner_mut: |option: *mut ()| unsafe {
                    (*(option as *mut Option<$type_id>))
                        .as_mut()
                        .map(|val| (val as *mut $type_id).cast::<()>())
                },

            };
        }
    };
}

impl<T: TypeReflection> TypeReflection for Option<T> {
    fn get_type(&self) -> TypeDefinition {
        T::get_type_def()
    }
    fn get_type_def() -> TypeDefinition {
        T::get_option_def()
    }
}
