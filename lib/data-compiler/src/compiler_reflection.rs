//! Compile an `OfflineType` to a `RuntimeType` using reflection
//!
use crate::compiler_api::CompilerError;

use lgn_data_offline::ResourcePathId;

use bincode::{DefaultOptions, Options};
use lgn_data_model::{TypeDefinition, TypeReflection};
use std::collections::HashSet;
use std::str::FromStr;

/// Convert a reflected `OfflineType` to a `RuntimeType` using reflection.
pub fn reflection_compile(
    offline_resource: &dyn TypeReflection,
    runtime_resource: &mut dyn TypeReflection,
) -> Result<(Vec<u8>, Option<HashSet<ResourcePathId>>), CompilerError> {
    // Read the Offline as a Serde_Json Value

    let mut buffer = Vec::new();
    let mut json = serde_json::Serializer::new(&mut buffer);
    let mut serializer = <dyn erased_serde::Serializer>::erase(&mut json);
    lgn_data_model::utils::serialize_property_by_name(offline_resource, "", &mut serializer)
        .map_err(|_e| CompilerError::CompilationError("Failed to read offline resource"))?;

    let str = std::str::from_utf8(buffer.as_slice())
        .map_err(|_e| CompilerError::CompilationError("Invalid serde_json serialization"))?;

    let mut values = serde_json::Value::from_str(str)
        .map_err(|_e| CompilerError::CompilationError("Invalid serde_json serialization"))?;

    // Process the Serde_Json value recursively and apply type conversion
    convert_json_value_to_runtime(runtime_resource.get_type(), &mut values).unwrap();

    // Apply Converted Values to Runtime instance
    let mut deserializer = <dyn erased_serde::Deserializer<'_>>::erase(values);
    lgn_data_model::utils::deserialize_property_by_name(runtime_resource, "", &mut deserializer)
        .map_err(|_e| {
            CompilerError::CompilationError("Failed to apply values to runtime resource")
        })?;

    let mut compiled_asset = Vec::new();

    let mut bincode_ser = bincode::Serializer::new(
        &mut compiled_asset,
        DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes(),
    );
    let mut serializer = <dyn erased_serde::Serializer>::erase(&mut bincode_ser);
    lgn_data_model::utils::serialize_property_by_name(runtime_resource, "", &mut serializer)
        .map_err(|_e| {
            CompilerError::CompilationError("Failed to serialize Runtime asset to bincode")
        })?;

    let resource_references = lgn_data_offline::extract_resource_dependencies(offline_resource);
    Ok((compiled_asset, resource_references))
}

fn convert_json_value_to_runtime(
    runtime_type_def: TypeDefinition,
    value: &mut serde_json::Value,
) -> anyhow::Result<()> {
    match runtime_type_def {
        TypeDefinition::None => {
            return Err(anyhow::anyhow!("error"));
        }
        TypeDefinition::BoxDyn(_box_dyn_descriptor) => {
            if let serde_json::Value::Object(object) = value {
                if object.len() == 1 {
                    for (k, v) in object {
                        if let serde_json::Value::Object(_sub_obj) = v {
                            let mut converted_value =
                                serde_json::Map::<String, serde_json::Value>::new();

                            let mut runtime_type = String::from("Runtime_");
                            runtime_type.push_str(k);
                            converted_value.insert(runtime_type, v.clone());
                            *value = serde_json::Value::Object(converted_value);
                            return Ok(());
                        }
                    }
                }
            }
        }

        TypeDefinition::Array(array_descriptor) => {
            if let serde_json::Value::Array(array) = value {
                array.iter_mut().try_for_each(|property| {
                    convert_json_value_to_runtime(array_descriptor.inner_type, property)
                })?;
            }
        }

        TypeDefinition::Primitive(primitive_descriptor) => {
            // Convert ResourcePathId to Runtime ReferenceType
            if primitive_descriptor
                .base_descriptor
                .type_name
                .ends_with("ReferenceType")
            {
                if let serde_json::Value::String(value_string) = value {
                    let res_path = ResourcePathId::from_str(value_string.as_str())
                        .map_err(|_e| anyhow::anyhow!("Invalid resourcePathId"))?;
                    *value = serde_json::to_value(res_path.resource_id())?;
                }
            }
        }
        TypeDefinition::Option(offline_option_descriptor) => {
            if !value.is_null() {
                convert_json_value_to_runtime(offline_option_descriptor.inner_type, value)?;
            }
        }
        TypeDefinition::Struct(struct_descriptor) => {
            if let serde_json::Value::Object(object) = value {
                struct_descriptor.fields.iter().try_for_each(
                    |runtime_field| -> anyhow::Result<()> {
                        if let Some(property_value) = object.get_mut(&runtime_field.field_name) {
                            convert_json_value_to_runtime(
                                runtime_field.field_type,
                                property_value,
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
