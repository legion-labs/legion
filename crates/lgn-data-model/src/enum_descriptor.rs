use crate::BaseDescriptor;
use std::collections::HashMap;

/// Define the reflection for a Enum type
pub struct EnumDescriptor {
    /// Base Descriptor of the type
    pub base_descriptor: BaseDescriptor,
    /// List of Attributes Enum
    pub attributes: Option<HashMap<String, String>>,
    /// Variants of the Enum
    pub variants: Vec<EnumVariantDescriptor>,
}

/// Descriptor of a Enum Variant
pub struct EnumVariantDescriptor {
    /// Name of the field
    pub variant_name: String,
    /// List of Attributes for the Field
    pub attributes: Option<HashMap<String, String>>,
}

/// Macro to implement Enum Descriptor for a type
#[macro_export]
macro_rules! implement_enum_descriptor {
    ($type_id:ty, $attributes:expr, $variants:expr) => {
        lazy_static::lazy_static! {
        static ref TYPE_DESCRIPTOR: $crate::EnumDescriptor = $crate::EnumDescriptor {
            base_descriptor : $crate::create_base_descriptor!($type_id, stringify!($type_id).into()),
            attributes : $attributes,
            variants : $variants
        };
    }
    };
}
