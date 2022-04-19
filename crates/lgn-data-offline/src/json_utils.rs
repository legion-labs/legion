use lgn_data_model::TypeDefinition;
use lgn_data_runtime::{extract_resource_dependencies, AssetRegistryError, Resource};

/// Write a Resource to a json stream
/// # Errors
/// Return `AssetRegistryError` on failure
pub fn to_json_writer(
    resource: &dyn Resource,
    mut writer: &mut dyn std::io::Write,
) -> Result<(), AssetRegistryError> {
    let base_values: Option<serde_json::Value> = {
        if let TypeDefinition::Struct(struct_def) = resource.get_type() {
            let base = (struct_def.new_instance)();
            let mut buffer = Vec::new();
            let mut json_ser = serde_json::Serializer::new(&mut buffer);
            let mut serializer = <dyn erased_serde::Serializer>::erase(&mut json_ser);
            lgn_data_model::utils::serialize_property_by_name(base.as_ref(), "", &mut serializer)?;
            let value = serde_json::from_slice::<serde_json::Value>(buffer.as_slice())
                .map_err(|err| lgn_data_model::ReflectionError::ErrorSerde(err.into()))?;

            Some(value)
        } else {
            None
        }
        //None
    };

    let mut resource_values = {
        let mut buffer = Vec::new();
        let mut json_ser = serde_json::Serializer::new(&mut buffer);
        let mut serializer = <dyn erased_serde::Serializer>::erase(&mut json_ser);
        lgn_data_model::utils::serialize_property_by_name(
            resource.as_reflect(),
            "",
            &mut serializer,
        )?;
        let mut value = serde_json::from_slice::<serde_json::Value>(buffer.as_slice())
            .map_err(|err| lgn_data_model::ReflectionError::ErrorSerde(err.into()))?;

        if let serde_json::Value::Object(object) = &mut value {
            let mut meta = object
                .remove("meta")
                .unwrap_or_else(|| serde_json::to_value(crate::get_meta(resource)).unwrap());

            if let Some(deps) = extract_resource_dependencies(resource.as_reflect()) {
                meta["dependencies"] = serde_json::json!(deps);
            } else {
                meta["dependencies"] = serde_json::json!([]);
            }

            serde_json::to_writer_pretty(&mut writer, &meta)
                .map_err(|err| AssetRegistryError::SerializationFailed("", err.to_string()))?;
            writeln!(writer)?;
        }
        value
    };

    if let Some(base_values) = base_values {
        if let Some(object) = resource_values.as_object_mut() {
            object.retain(|property_name, cur_value| {
                base_values
                    .get(property_name.as_str())
                    .map_or(false, |base_value| base_value != cur_value)
            });
        }
    }

    serde_json::to_writer_pretty(&mut writer, &resource_values)
        .map_err(|err| AssetRegistryError::SerializationFailed("", err.to_string()))?;
    Ok(())
}
