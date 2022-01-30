use crate::{BaseDescriptor, FieldDescriptor, TypeReflection};
/// Define the reflection of a Struct
pub struct StructDescriptor {
    /// Base Descriptor for the Struct type
    pub base_descriptor: BaseDescriptor,
    /// Return a new instance using name
    pub new_instance: fn() -> Box<dyn TypeReflection>,
    /// Fields of the Struct
    pub fields: Vec<FieldDescriptor>,
}

/// Macro to implement Primitive Descriptor for a primtive type
#[macro_export]
macro_rules! implement_struct_descriptor {
    ($type_id:ty, $field:expr) => {
        lazy_static::lazy_static! {
                static ref TYPE_DESCRIPTOR: $crate::StructDescriptor = $crate::StructDescriptor {
                base_descriptor : $crate::create_base_descriptor!($type_id, stringify!($type_id).into()),
                new_instance : || Box::new(<$type_id>::default()),
                fields: $field
            };
        }
    };
}
