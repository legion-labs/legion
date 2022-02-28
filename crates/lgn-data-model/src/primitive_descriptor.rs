use crate::BaseDescriptor;

/// Define the reflection for a Primitive type
pub struct PrimitiveDescriptor {
    /// Base Descriptor of the type
    pub base_descriptor: BaseDescriptor,
}

/// Macro to implement primitive type definition
#[macro_export]
macro_rules! implement_primitive_type_def {
    ($type_name:ty) => {
        impl $crate::TypeReflection for $type_name {
            fn get_type(&self) -> $crate::TypeDefinition {
                <$type_name>::get_type_def()
            }

            fn get_type_def() -> $crate::TypeDefinition {
                lazy_static::lazy_static! {
                    static ref TYPE_DESCRIPTOR: $crate::PrimitiveDescriptor = $crate::PrimitiveDescriptor {
                        base_descriptor : $crate::create_base_descriptor!($type_name, stringify!($type_name).into(),
                        Result::<$type_name, $crate::ReflectionError>::Ok(<$type_name>::default()))
                    };
                }
                $crate::TypeDefinition::Primitive(&TYPE_DESCRIPTOR)
            }
            fn get_option_def() -> $crate::TypeDefinition {
                $crate::implement_option_descriptor!($type_name);
                $crate::TypeDefinition::Option(&OPTION_DESCRIPTOR)
            }

            fn get_array_def() -> $crate::TypeDefinition {
                $crate::implement_array_descriptor!($type_name);
                $crate::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
            }
        }
    };

    ($type_name:ty, $default_expr:expr) => {
        impl $crate::TypeReflection for $type_name {
            fn get_type(&self) -> $crate::TypeDefinition {
                <$type_name>::get_type_def()
            }

            fn get_type_def() -> $crate::TypeDefinition {
                lazy_static::lazy_static! {
                    static ref TYPE_DESCRIPTOR: $crate::PrimitiveDescriptor = $crate::PrimitiveDescriptor {
                        base_descriptor : $crate::create_base_descriptor!($type_name, stringify!($type_name).into(),
                        $default_expr)
                    };
                }
                $crate::TypeDefinition::Primitive(&TYPE_DESCRIPTOR)
            }
            fn get_option_def() -> $crate::TypeDefinition {
                $crate::implement_option_descriptor!($type_name);
                $crate::TypeDefinition::Option(&OPTION_DESCRIPTOR)
            }

            fn get_array_def() -> $crate::TypeDefinition {
                $crate::implement_array_descriptor!($type_name);
                $crate::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
            }
        }
    };
}

/// Macro to implement primitive type definition
#[macro_export]
macro_rules! implement_reference_type_def {
    ($type_name:ident, $inner_type:ty) => {
        /// Reference Type for Texture
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Clone)]
        pub struct $type_name(lgn_data_runtime::Reference<$inner_type>);
        impl $type_name {
            /// Expose internal id
            pub fn id(&self) -> lgn_data_runtime::ResourceTypeAndId {
                self.0.id()
            }
        }

        impl $crate::TypeReflection for $type_name {
            fn get_type(&self) -> $crate::TypeDefinition {
                <$type_name>::get_type_def()
            }

            fn get_type_def() -> $crate::TypeDefinition {
                lazy_static::lazy_static! {
                    static ref TYPE_DESCRIPTOR: $crate::PrimitiveDescriptor = $crate::PrimitiveDescriptor {
                        base_descriptor : $crate::create_base_descriptor!($type_name, stringify!($type_name).into(),
                                            Err($crate::ReflectionError::UnsupportedDefault(stringify!($type_name))))
                    };
                }

                $crate::TypeDefinition::Primitive(&TYPE_DESCRIPTOR)
            }
            fn get_option_def() -> $crate::TypeDefinition {
                $crate::implement_option_descriptor!($type_name);
                $crate::TypeDefinition::Option(&OPTION_DESCRIPTOR)
            }

            fn get_array_def() -> $crate::TypeDefinition {
                $crate::implement_array_descriptor!($type_name);
                $crate::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
            }
        }
    };
}

// Implementation of the basic primitive types
implement_primitive_type_def!(bool);
implement_primitive_type_def!(usize);
implement_primitive_type_def!(u8);
implement_primitive_type_def!(i8);
implement_primitive_type_def!(u16);
implement_primitive_type_def!(u32);
implement_primitive_type_def!(i32);
implement_primitive_type_def!(f32);
implement_primitive_type_def!(i64);
implement_primitive_type_def!(u64);
implement_primitive_type_def!(f64);
implement_primitive_type_def!(String);

use lgn_math::prelude::*;
implement_primitive_type_def!(Vec2);
implement_primitive_type_def!(Vec3);
implement_primitive_type_def!(Vec4);
implement_primitive_type_def!(Quat);
