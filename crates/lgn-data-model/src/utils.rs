use thiserror::Error;

use crate::{
    type_reflection::{TypeDefinition, TypeReflection},
    BaseDescriptor,
};

/// Internal struct to store `ReflectedPtr`
pub struct ReflectedPtr<'a> {
    /// ReflectedPtr base
    pub base: *const (),
    /// ReflectedPtr type
    pub type_def: TypeDefinition,
    /// Base Descriptor of the type
    pub base_descriptor: &'a BaseDescriptor,
    _covariant: std::marker::PhantomData<&'a ()>,
}

/// Internal struct to store `ReflectedPtrMut`
pub struct ReflectedPtrMut<'a> {
    /// ReflectedPtr base
    pub base: *mut (),
    /// ReflectedPtr type
    pub type_def: TypeDefinition,
    /// Base Descriptor of the type
    pub base_descriptor: &'a BaseDescriptor,
    _covariant: std::marker::PhantomData<&'a ()>,
}

/// Error for Reflection system
#[allow(missing_docs)]
#[derive(Error, Debug)]
pub enum ReflectionError {
    #[error("Invalid TypeDescriptor in path '{0}'")]
    InvalidTypeDescriptor(String),

    #[error("Error parsing array index in path '{0}' on ArrayDescriptor '{1}'")]
    ParsingArrayIndex(String, String),

    #[error("Array key '{0}' on ArrayDescriptor '{1}'")]
    ArrayKeyNotFound(String, String),

    #[error("Option field '{0}' not found on empty OptionDescriptor '{1}'")]
    FieldNotFoundOnEmptyOption(String, String),

    #[error("Field '{0}' not found on StructDescriptor '{1}'")]
    FieldNotFoundOnStruct(String, String),

    #[error("Invalid Property path '{0}' on StructDescriptor '{1}'")]
    InvalidPathForStruct(String, String),

    #[error("Invalid field type for '{0}'")]
    InvalidFieldType(String),

    #[error("Invalid Utf8 property: {0}'")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),

    #[error("Serialization error: {0}")]
    ErrorSerde(#[from] serde_json::Error),

    #[error("Serialization error: {0}")]
    ErrorErasedSerde(#[from] erased_serde::Error),

    /// Error when accessing out of bounds index
    #[error("Invalid array index {0} on ArrayDescriptor '{1}'")]
    InvalidArrayIndex(usize, &'static str),

    /// Error when accessing out of bounds index
    #[error("Value not found on ArrayDescriptor '{0}'")]
    InvalidArrayValue(&'static str),

    /// Generic error when there's no context
    #[error("'{0}'")]
    Generic(String),

    #[error("{field_path}, {inner_error}")]
    FieldError {
        field_path: String,
        inner_error: String,
    },

    /// Error when type is unknown
    #[error("Type '{0}' not found")]
    TypeNotFound(String),
}

/// Deserialize a property by reflection
pub fn deserialize_property_by_name<'de>(
    object: &mut dyn TypeReflection,
    path: &str,
    deserializer: &mut dyn erased_serde::Deserializer<'de>,
) -> Result<(), ReflectionError> {
    find_property(object, path).and_then(|property| unsafe {
        (property.base_descriptor.dynamic_deserialize)(property.base as *mut (), deserializer)
    })
}

/// Serialize a property by reflection
pub fn serialize_property_by_name(
    object: &dyn TypeReflection,
    path: &str,
    serializer: &mut dyn erased_serde::Serializer,
) -> Result<(), ReflectionError> {
    find_property(object, path).and_then(|property| unsafe {
        (property.base_descriptor.dynamic_serialize)(property.base, serializer)
    })
}

/// Get Property from a Path
pub fn find_property<'a>(
    base: &dyn TypeReflection,
    path: &str,
) -> Result<ReflectedPtr<'a>, ReflectionError> {
    internal_find_property(
        (base as *const dyn TypeReflection).cast::<()>(),
        base.get_type(),
        path,
    )
}

/// Get Property from a Path
pub fn find_property_mut<'a>(
    base: &mut dyn TypeReflection,
    path: &str,
) -> Result<ReflectedPtrMut<'a>, ReflectionError> {
    let out = internal_find_property(
        (base as *const dyn TypeReflection).cast::<()>(),
        base.get_type(),
        path,
    )?;

    Ok(ReflectedPtrMut {
        base: out.base as *mut (),
        type_def: out.type_def,
        base_descriptor: out.base_descriptor,
        _covariant: std::marker::PhantomData,
    })
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
fn internal_find_property<'a>(
    base: *const (),
    type_def: TypeDefinition,
    path: &str,
) -> Result<ReflectedPtr<'a>, ReflectionError> {
    match type_def {
        TypeDefinition::None => Err(ReflectionError::InvalidTypeDescriptor(path.into())),
        TypeDefinition::BoxDyn(box_dyn_descriptor) => {
            let sub_type = unsafe { (box_dyn_descriptor.get_inner_type)(base) };
            let sub_base = unsafe { (box_dyn_descriptor.get_inner)(base) };
            internal_find_property(sub_base, sub_type, path)
        }

        TypeDefinition::Array(array_descriptor) => {
            let mut rest_of_path = path;

            if rest_of_path.is_empty() {
                return Ok(ReflectedPtr {
                    base,
                    type_def,
                    base_descriptor: &array_descriptor.base_descriptor,
                    _covariant: std::marker::PhantomData,
                });
            }

            if path.starts_with('[') {
                if let Some(end_brace) = path.find(']') {
                    rest_of_path = path[(end_brace + 1)..].trim_start_matches('.');
                    // Extract the index or 'key'
                    let index_identifier = &path[1..end_brace];
                    if index_identifier.chars().next().map(char::is_numeric) == Some(true) {
                        let parsed_index = index_identifier.parse::<u32>().map_err(|_err| {
                            ReflectionError::ParsingArrayIndex(
                                index_identifier.into(),
                                array_descriptor.base_descriptor.type_name.clone(),
                            )
                        })?;
                        let element_base =
                            unsafe { (array_descriptor.get)(base, parsed_index as usize) }?;
                        return internal_find_property(
                            element_base,
                            array_descriptor.inner_type,
                            rest_of_path,
                        );
                    } else if let TypeDefinition::BoxDyn(box_desc) = array_descriptor.inner_type {
                        let count = unsafe { (array_descriptor.len)(base) };
                        for index in 0..count {
                            let element_base =
                                unsafe { (array_descriptor.get)(base, index as usize) }?;
                            let inner_type = unsafe { (box_desc.get_inner_type)(element_base) };
                            if inner_type.get_type_name().to_lowercase()
                                == index_identifier.to_lowercase()
                            {
                                return internal_find_property(
                                    element_base,
                                    array_descriptor.inner_type,
                                    rest_of_path,
                                );
                            }
                        }
                        return Err(ReflectionError::ArrayKeyNotFound(
                            index_identifier.into(),
                            array_descriptor.base_descriptor.type_name.clone(),
                        ));
                    }
                }
            }

            Err(ReflectionError::ParsingArrayIndex(
                path.into(),
                array_descriptor.base_descriptor.type_name.clone(),
            ))
        }

        TypeDefinition::Primitive(primitive_descriptor) => Ok(ReflectedPtr {
            base,
            type_def,
            base_descriptor: &primitive_descriptor.base_descriptor,
            _covariant: std::marker::PhantomData,
        }),

        TypeDefinition::Enum(enum_descriptor) => Ok(ReflectedPtr {
            base,
            type_def,
            base_descriptor: &enum_descriptor.base_descriptor,
            _covariant: std::marker::PhantomData,
        }),

        TypeDefinition::Option(option_descriptor) => {
            if path.is_empty() {
                Ok(ReflectedPtr {
                    base,
                    type_def,
                    base_descriptor: &option_descriptor.base_descriptor,
                    _covariant: std::marker::PhantomData,
                })
            } else if let Some(value_base) = unsafe { (option_descriptor.get_inner)(base) } {
                internal_find_property(value_base, option_descriptor.inner_type, path)
            } else {
                Err(ReflectionError::FieldNotFoundOnEmptyOption(
                    path.into(),
                    option_descriptor.base_descriptor.type_name.clone(),
                ))
            }
        }
        TypeDefinition::Struct(struct_descriptor) => {
            if path.is_empty() {
                return Ok(ReflectedPtr {
                    base,
                    type_def,
                    base_descriptor: &struct_descriptor.base_descriptor,
                    _covariant: std::marker::PhantomData,
                });
            }

            let mut split_path = path.split(&['[', '.'][..]);
            split_path.next().map_or_else(
                || {
                    Err(ReflectionError::InvalidPathForStruct(
                        path.into(),
                        struct_descriptor.base_descriptor.type_name.clone(),
                    ))
                },
                |field_name| {
                    struct_descriptor
                        .fields
                        .iter()
                        .filter(|field| field.field_name == field_name)
                        .map(|field| {
                            let field_base =
                                unsafe { base.cast::<u8>().add(field.offset).cast::<()>() };
                            internal_find_property(
                                field_base,
                                field.field_type,
                                path[field_name.len()..].trim_start_matches('.'),
                            )
                            .map_err(|err| {
                                ReflectionError::FieldError {
                                    field_path: format!(
                                        "{}.{}",
                                        struct_descriptor.base_descriptor.type_name,
                                        field.field_name
                                    ),
                                    inner_error: err.to_string(),
                                }
                            })
                        })
                        .next()
                        .unwrap_or_else(|| {
                            Err(ReflectionError::FieldNotFoundOnStruct(
                                field_name.into(),
                                struct_descriptor.base_descriptor.type_name.clone(),
                            ))
                        })
                },
            )
        }
    }
}
