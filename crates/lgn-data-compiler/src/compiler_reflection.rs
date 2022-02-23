//! Compile an `OfflineType` to a `RuntimeType` using reflection
//!
use crate::compiler_api::CompilerError;

use lgn_data_offline::ResourcePathId;

use bincode::{DefaultOptions, Options};
use lgn_data_model::{ReflectionError, TypeDefinition, TypeReflection};
use std::str::FromStr;

/// Convert a reflected `OfflineType` to a `RuntimeType` using reflection.
pub fn reflection_compile(
    offline_resource: &dyn TypeReflection,
    runtime_resource: &mut dyn TypeReflection,
) -> Result<(Vec<u8>, Option<Vec<ResourcePathId>>), CompilerError> {
    // Read the Offline as a Serde_Json Value

    let mut buffer = Vec::new();
    let mut json = serde_json::Serializer::new(&mut buffer);
    let mut serializer = <dyn erased_serde::Serializer>::erase(&mut json);
    lgn_data_model::utils::serialize_property_by_name(offline_resource, "", &mut serializer)?;

    let mut values = serde_json::from_slice(buffer.as_slice())?;

    // Process the Serde_Json value recursively and apply type conversion
    convert_json_value_to_runtime(
        (offline_resource as *const dyn TypeReflection).cast::<()>(),
        offline_resource.get_type(),
        &mut values,
    )?;

    // Apply Converted Values to Runtime instance
    let mut deserializer = <dyn erased_serde::Deserializer<'_>>::erase(values);
    lgn_data_model::utils::deserialize_property_by_name(runtime_resource, "", &mut deserializer)?;

    let mut compiled_asset = Vec::new();

    let mut bincode_ser = bincode::Serializer::new(
        &mut compiled_asset,
        DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes(),
    );
    let mut serializer = <dyn erased_serde::Serializer>::erase(&mut bincode_ser);
    lgn_data_model::utils::serialize_property_by_name(runtime_resource, "", &mut serializer)?;

    let resource_references = lgn_data_offline::extract_resource_dependencies(offline_resource);
    Ok((compiled_asset, resource_references))
}

fn convert_json_value_to_runtime(
    source: *const (),
    offline_type_def: TypeDefinition,
    json_value: &mut serde_json::Value,
) -> Result<(), ReflectionError> {
    match offline_type_def {
        TypeDefinition::None => {
            return Err(ReflectionError::InvalidTypeDescriptor("NullType".into()));
        }
        TypeDefinition::BoxDyn(box_dyn_descriptor) => {
            // For BoxDyn, pipe directly to the inner type
            let sub_base = (box_dyn_descriptor.get_inner)(source);
            let sub_type = (box_dyn_descriptor.get_inner_type)(source);

            if let serde_json::Value::Object(object) = json_value {
                if object.len() == 1 {
                    for (k, v) in object {
                        if let serde_json::Value::Object(_sub_obj) = v {
                            convert_json_value_to_runtime(sub_base, sub_type, v)?;
                            *json_value =
                                serde_json::json!({ format!("Runtime_{}", k): v.clone() });
                            return Ok(());
                        }
                    }
                }
            }
        }

        TypeDefinition::Array(array_descriptor) => {
            if let serde_json::Value::Array(array) = json_value {
                for index in 0..(array_descriptor.len)(source) {
                    let item_base = (array_descriptor.get)(source, index)?;
                    let item_type_def = array_descriptor.inner_type;
                    if let Some(element_value) = array.get_mut(index) {
                        convert_json_value_to_runtime(item_base, item_type_def, element_value)?;
                    }
                }
            }
        }

        TypeDefinition::Enum(_enum_descriptor) => {
            // Keep enum as is
        }

        TypeDefinition::Primitive(primitive_descriptor) => {
            // Convert ResourcePathId to Runtime ReferenceType
            if primitive_descriptor.base_descriptor.type_name == "ResourcePathId" {
                if let serde_json::Value::String(value_string) = json_value {
                    let res_path =
                        ResourcePathId::from_str(value_string.as_str()).map_err(|err| {
                            ReflectionError::Generic(format!(
                                "Invalid ResourcePathId '{}': {}",
                                value_string, err
                            ))
                        })?;
                    *json_value = serde_json::to_value(res_path.resource_id())?;
                }
            }
        }
        TypeDefinition::Option(offline_option_descriptor) => {
            if let Some(value_base) = unsafe { (offline_option_descriptor.get_inner)(source) } {
                convert_json_value_to_runtime(
                    value_base,
                    offline_option_descriptor.inner_type,
                    json_value,
                )?;
            }
        }
        TypeDefinition::Struct(struct_descriptor) => {
            if let serde_json::Value::Object(object) = json_value {
                struct_descriptor.fields.iter().try_for_each(
                    |offline_field| -> Result<(), ReflectionError> {
                        if let Some(field_value) = object.get_mut(&offline_field.field_name) {
                            let field_base = unsafe {
                                source.cast::<u8>().add(offline_field.offset).cast::<()>()
                            };

                            convert_json_value_to_runtime(
                                field_base,
                                offline_field.field_type,
                                field_value,
                            )?;
                        }
                        Ok(())
                    },
                )?;
            }
        }
    }
    Ok(())
}
