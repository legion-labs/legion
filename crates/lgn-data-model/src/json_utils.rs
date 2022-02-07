use crate::type_reflection::{TypeDefinition, TypeReflection};
use crate::utils::{deserialize_property_by_name, serialize_property_by_name, ReflectionError};

/// Read a Property as a json value
pub fn get_property_as_json_string(
    object: &dyn TypeReflection,
    path: &str,
) -> Result<String, ReflectionError> {
    let mut buffer = Vec::new();
    let mut json = serde_json::Serializer::new(&mut buffer);
    let mut serializer = <dyn erased_serde::Serializer>::erase(&mut json);
    serialize_property_by_name(object, path, &mut serializer)?;
    Ok(String::from_utf8(buffer)?)
}

/// Write a Property as a json value
pub fn set_property_from_json_string(
    object: &mut dyn TypeReflection,
    path: &str,
    json_value: &str,
) -> Result<(), ReflectionError> {
    let mut json = serde_json::Deserializer::from_str(json_value);
    let mut deserializer = <dyn erased_serde::Deserializer<'_>>::erase(&mut json);
    deserialize_property_by_name(object, path, &mut deserializer)
}

/// Serialize a `ReflectedData` to Json, appling delta from parent data
pub fn reflection_save_relative_json<T: Sized + serde::Serialize>(
    entity: &T,
    base_entity: &T,
) -> Result<serde_json::Value, ReflectionError> {
    let mut values = serde_json::to_value(&entity)?;
    let base_values = serde_json::to_value(&base_entity)?;

    if let Some(object) = values.as_object_mut() {
        object.retain(|property_name, cur_value| {
            base_values
                .get(property_name.as_str())
                .map_or(false, |base_value| base_value != cur_value)
        });
    }

    Ok(values)
}

/// Apply json edit an object,
pub fn reflection_apply_json_edit<T: TypeReflection>(
    object: &mut T,
    values: &serde_json::Value,
) -> Result<(), ReflectionError> {
    internal_apply_json_edit(
        Some((object as *mut dyn TypeReflection).cast::<()>()),
        object.get_type(),
        values,
    )?;
    Ok(())
}

// Recursively serialize the json_value using the Reflection descriptor
fn internal_apply_json_edit(
    target: Option<*mut ()>,
    offline_type_def: TypeDefinition,
    json_value: &serde_json::Value,
) -> Result<serde_json::Value, ReflectionError> {
    match offline_type_def {
        TypeDefinition::None => {
            return Err(ReflectionError::InvalidFieldType(
                offline_type_def.get_type_name().into(),
            ));
        }
        TypeDefinition::BoxDyn(box_dyn_descriptor) => {
            // Read the 'typetag' map
            if let serde_json::Value::Object(object) = json_value {
                if let Some((k, value)) = object.iter().next() {
                    // Find the dyn Type descriptor using factory
                    if let Some(instance_type) = (box_dyn_descriptor.find_type)(k.as_str()) {
                        let result_json = internal_apply_json_edit(None, instance_type, value)?;
                        return Ok(serde_json::json!({ k: result_json }));
                    }
                    return Err(ReflectionError::TypeNotFound(k.clone()));
                }
            }
        }
        TypeDefinition::Array(array_descriptor) => {
            if let serde_json::Value::Array(array) = json_value {
                let array_target = target.unwrap();
                // Reset array using descriptor
                unsafe {
                    (array_descriptor.clear)(array_target);
                }

                for value in array.iter() {
                    // For each element, apply the values and return the merge json
                    match internal_apply_json_edit(None, array_descriptor.inner_type, value) {
                        Ok(merged_json) => {
                            // Add the new element into the array using the merged_json result
                            unsafe {
                                (array_descriptor.insert_element)(
                                    array_target,
                                    None,
                                    &mut <dyn erased_serde::Deserializer<'_>>::erase(&merged_json),
                                )?;
                            }
                        }
                        Err(ReflectionError::TypeNotFound(type_name)) => {
                            // Ignore missing type
                            lgn_tracing::warn!("Skipping unknown type: {}", type_name);
                        }
                        Err(error) => return Err(error),
                    }
                }
            }
        }

        TypeDefinition::Enum(enum_descriptor) => {
            // If there's a target, serialize in it directly, else return the json_value as is to be add to array or option
            if let Some(target) = target {
                let mut deserializer = <dyn erased_serde::Deserializer<'_>>::erase(json_value);
                unsafe {
                    (enum_descriptor.base_descriptor.dynamic_deserialize)(
                        target,
                        &mut deserializer,
                    )?;
                }
                return Ok(serde_json::Value::Null);
            }
            return Ok(json_value.clone());
        }

        TypeDefinition::Primitive(primitive_descriptor) => {
            // If there's a target, serialize in it directly, else return the json_value as is to be add to array or option
            if let Some(target) = target {
                let mut deserializer = <dyn erased_serde::Deserializer<'_>>::erase(json_value);
                unsafe {
                    (primitive_descriptor.base_descriptor.dynamic_deserialize)(
                        target,
                        &mut deserializer,
                    )?;
                }
                return Ok(serde_json::Value::Null);
            }
            return Ok(json_value.clone());
        }

        TypeDefinition::Option(offline_option_descriptor) => {
            let option_value = if !json_value.is_null() {
                // Recursively process the option value
                internal_apply_json_edit(None, offline_option_descriptor.inner_type, json_value)?
            } else {
                serde_json::Value::Null
            };

            // If there's a 'target', apply the value
            if let Some(target) = target {
                let mut deserializer = <dyn erased_serde::Deserializer<'_>>::erase(&option_value);
                unsafe {
                    (offline_option_descriptor
                        .base_descriptor
                        .dynamic_deserialize)(target, &mut deserializer)?;
                }
                return Ok(serde_json::Value::Null);
            }
            return Ok(option_value);
        }
        TypeDefinition::Struct(struct_descriptor) => {
            // If there's already a target, process all its fields
            if let Some(target) = target {
                if let serde_json::Value::Object(object) = json_value {
                    struct_descriptor.fields.iter().try_for_each(
                        |offline_field| -> Result<(), ReflectionError> {
                            if let Some(field_value) = object.get(&offline_field.field_name) {
                                let field_base = unsafe {
                                    target.cast::<u8>().add(offline_field.offset).cast::<()>()
                                };
                                internal_apply_json_edit(
                                    Some(field_base),
                                    offline_field.field_type,
                                    field_value,
                                )
                                .map_err(|err| {
                                    ReflectionError::FieldError {
                                        field_path: format!(
                                            "{}.{}",
                                            struct_descriptor.base_descriptor.type_name,
                                            offline_field.field_name
                                        ),
                                        inner_error: err.to_string(),
                                    }
                                })?;
                            }
                            Ok(())
                        },
                    )?;
                }
            } else {
                // If the target doesn't exists (array/option init), create a default one and recursively process it
                let mut new_struct = (struct_descriptor.new_instance)();
                let ty: &mut dyn TypeReflection = new_struct.as_mut();
                internal_apply_json_edit(
                    Some((ty as *mut dyn TypeReflection).cast::<()>()),
                    ty.get_type(),
                    json_value,
                )?;

                // Re-serialize the entire struct to generated a merge_json to return to the parent array/option
                let mut buffer = Vec::new();
                let mut json = serde_json::Serializer::new(&mut buffer);
                let mut serializer = <dyn erased_serde::Serializer>::erase(&mut json);
                unsafe {
                    (struct_descriptor.base_descriptor.dynamic_serialize)(
                        (ty as *const dyn TypeReflection).cast::<()>(),
                        &mut serializer,
                    )?;
                }
                return Ok(serde_json::from_slice::<serde_json::Value>(
                    buffer.as_slice(),
                )?);
            }
        }
    }
    Ok(serde_json::Value::Null)
}
