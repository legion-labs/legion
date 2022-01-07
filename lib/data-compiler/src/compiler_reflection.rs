//! Compile an `OfflineType` to a `RuntimeType` using reflection
//!
use crate::compiler_api::CompilerError;

use lgn_data_offline::ResourcePathId;

use bincode::{DefaultOptions, Options};
use lgn_data_model::collector::{ItemInfo, PropertyCollector};
use lgn_data_model::{TypeDefinition, TypeReflection};
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

    let resource_references = extract_resource_dependencies(offline_resource);
    Ok((compiled_asset, resource_references))
}

fn extract_resource_dependencies(object: &dyn TypeReflection) -> Option<Vec<ResourcePathId>> {
    struct ExtractResourcePathId {
        output: Vec<ResourcePathId>,
    }

    impl PropertyCollector for ExtractResourcePathId {
        type Item = Option<Self>;
        fn new_item(item_info: &ItemInfo<'_>) -> anyhow::Result<Self::Item> {
            if let TypeDefinition::Primitive(primitive_descriptor) = item_info.type_def {
                if primitive_descriptor.base_descriptor.type_name == "ResourcePathId" {
                    let mut output = Vec::new();
                    let mut json = serde_json::Serializer::new(&mut output);
                    let mut serializer = <dyn erased_serde::Serializer>::erase(&mut json);
                    unsafe {
                        (primitive_descriptor.base_descriptor.dynamic_serialize)(
                            item_info.base,
                            &mut serializer,
                        )?;
                    }

                    let path = String::from_utf8(output)?;
                    if let Ok(res_id) = ResourcePathId::from_str(path.as_str()) {
                        return Ok(Some(Self {
                            output: vec![res_id],
                        }));
                    }
                }
            }
            Ok(None)
        }
        fn add_child(parent: &mut Self::Item, child: Self::Item) {
            if let Some(child) = child {
                parent
                    .get_or_insert(Self { output: Vec::new() })
                    .output
                    .extend(child.output);
            }
        }
    }

    if let Ok(Some(total)) =
        lgn_data_model::collector::collect_properties::<ExtractResourcePathId>(object)
    {
        return Some(total.output);
    }
    None
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

        TypeDefinition::Primitive(_primitive_descriptor) => {}
        TypeDefinition::Option(_offline_option_descriptor) => {}
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
