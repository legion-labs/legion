/// Trait that implement reflection functions
pub trait TypeReflection {
    /// Return the `TypeDefinition` of the instance
    fn get_type(&self) -> TypeDefinition;

    /// Return the `TypeDefinition` for a Type
    fn get_type_def() -> TypeDefinition
    where
        Self: Sized;

    /// Return the `TypeDefinition` for a Option<Type>
    fn get_option_def() -> TypeDefinition
    where
        Self: Sized,
    {
        TypeDefinition::None
    }
    /// Return the `ArrayDefinition` for a Vec<Type>
    fn get_array_def() -> TypeDefinition
    where
        Self: Sized,
    {
        TypeDefinition::None
    }
}

/// Type Definition
#[derive(Clone, Copy)]
pub enum TypeDefinition {
    /// Invalid Type
    None,
    /// Primitive Type
    Primitive(&'static crate::PrimitiveDescriptor),
    /// Struct Type
    Struct(&'static crate::StructDescriptor),
    /// Array Type
    Array(&'static crate::ArrayDescriptor),
    /// Option Type
    Option(&'static crate::OptionDescriptor),
    /// Box<dyn XXX> Type
    BoxDyn(&'static crate::BoxDynDescriptor),
    /// Enum Type
    Enum(&'static crate::EnumDescriptor),
}

impl TypeDefinition {
    /// Return the name of the type
    pub fn get_type_name(&self) -> &str {
        match *self {
            Self::Array(array_descriptor) => array_descriptor.base_descriptor.type_name.as_str(),
            Self::Struct(struct_descriptor) => struct_descriptor.base_descriptor.type_name.as_str(),
            Self::Primitive(primitive_descriptor) => {
                primitive_descriptor.base_descriptor.type_name.as_str()
            }
            Self::Option(option_descriptor) => option_descriptor.base_descriptor.type_name.as_str(),
            Self::BoxDyn(box_dyn_descriptor) => {
                box_dyn_descriptor.base_descriptor.type_name.as_str()
            }
            Self::Enum(enum_descriptor) => enum_descriptor.base_descriptor.type_name.as_str(),
            Self::None => "None",
        }
    }

    /// Return the name of the type
    pub fn serialize_default(
        &self,
        serializer: &mut dyn ::erased_serde::Serializer,
    ) -> Result<(), crate::ReflectionError> {
        match *self {
            Self::Array(array_descriptor) => {
                (array_descriptor.base_descriptor.serialize_new_instance)(serializer)
            }
            Self::Struct(struct_descriptor) => {
                (struct_descriptor.base_descriptor.serialize_new_instance)(serializer)
            }
            Self::Primitive(primitive_descriptor) => {
                (primitive_descriptor.base_descriptor.serialize_new_instance)(serializer)
            }
            Self::Option(option_descriptor) => {
                (option_descriptor.base_descriptor.serialize_new_instance)(serializer)
            }
            Self::BoxDyn(box_dyn_descriptor) => {
                (box_dyn_descriptor.base_descriptor.serialize_new_instance)(serializer)
            }
            Self::Enum(enum_descriptor) => {
                (enum_descriptor.base_descriptor.serialize_new_instance)(serializer)
            }
            Self::None => Err(crate::ReflectionError::InvalidTypeDescriptor("None".into())),
        }
    }
}
