use crate::{BaseDescriptor, FieldDescriptor};
/// Define the reflection of a Struct
pub struct StructDescriptor {
    /// Base Descriptor for the Struct type
    pub base_descriptor: BaseDescriptor,
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
                fields: $field
            };
        }
    };
}
