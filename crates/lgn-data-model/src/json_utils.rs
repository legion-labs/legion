use crate::type_reflection::TypeReflection;
use crate::utils::ReflectionError;
use crate::utils::{deserialize_property_by_name, serialize_property_by_name};

/// Read a Property as a json value
pub fn get_property_as_json_string(
    object: &dyn TypeReflection,
    path: &str,
) -> anyhow::Result<String> {
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
) -> anyhow::Result<()> {
    let mut json = serde_json::Deserializer::from_str(json_value);
    let mut deserializer = <dyn erased_serde::Deserializer<'_>>::erase(&mut json);
    deserialize_property_by_name(object, path, &mut deserializer)
}

/// Serialize a `ReflectedData` to Json, appling delta from parent data
pub fn reflection_save_relative_json<T: Sized + serde::Serialize>(
    entity: &T,
    base_entity: &T,
) -> anyhow::Result<serde_json::Value> {
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
) -> anyhow::Result<()> {
    if let serde_json::Value::Object(json_object) = values {
        json_object
            .iter()
            .try_for_each(|(key, value)| -> anyhow::Result<()> {
                let mut erased_de = <dyn erased_serde::Deserializer<'_>>::erase(value);
                match deserialize_property_by_name(object, key, &mut erased_de) {
                    Ok(_) => Ok(()),
                    Err(err) => {
                        // Ignore InvalidPath when applying json
                        if let Some(ReflectionError::FieldNotFoundOnStruct(_, _)) =
                            err.downcast_ref::<ReflectionError>()
                        {
                            Ok(())
                        } else {
                            Err(err)
                        }
                    }
                }
            })?;
    }
    Ok(())
}
