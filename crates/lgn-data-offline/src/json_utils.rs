use std::sync::Arc;

use lgn_data_model::{ReflectionError, TypeDefinition};
use lgn_data_runtime::{extract_resource_dependencies, prelude::*};

/// Implement a default `ResourceInstaller` using `from_json_reader` interface
pub struct JsonInstaller<T: Resource> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Resource + Default> JsonInstaller<T> {
    /// Create a new Json installer for a specific Resource type
    pub fn create() -> Arc<dyn ResourceInstaller> {
        Arc::new(Self {
            _phantom: std::marker::PhantomData::<T>,
        })
    }
}

#[async_trait::async_trait]
impl<T: Resource + Default> ResourceInstaller for JsonInstaller<T> {
    async fn install_from_stream(
        &self,
        resource_id: ResourceTypeAndId,
        request: &mut LoadRequest,
        reader: &mut AssetRegistryReader,
    ) -> Result<HandleUntyped, AssetRegistryError> {
        let new_resource = from_json_reader::<T>(reader).await?;
        let handle = request
            .asset_registry
            .set_resource(resource_id, new_resource)?;
        Ok(handle)
    }
}

pub async fn from_json_reader<T: Resource>(
    reader: &mut AssetRegistryReader,
) -> Result<Box<T>, AssetRegistryError> {
    let resource = from_json_reader_untyped(reader).await?;
    if resource.is::<T>() {
        let raw: *mut dyn lgn_data_runtime::Resource = Box::into_raw(resource);
        #[allow(unsafe_code, clippy::cast_ptr_alignment)]
        let boxed_asset = unsafe { Box::from_raw(raw.cast::<T>()) };
        return Ok(boxed_asset);
    }
    Err(lgn_data_runtime::AssetRegistryError::Generic(
        "invalid type".into(),
    ))
}

/// Create a Resource from a json stream
/// # Errors
/// Return `AssetRegistryError` on failure
pub async fn from_json_reader_untyped(
    reader: &mut AssetRegistryReader,
) -> Result<Box<dyn Resource>, AssetRegistryError> {
    use tokio::io::AsyncReadExt;
    let mut buffer = Vec::<u8>::new();
    reader.read_to_end(&mut buffer).await?;
    let mut stream =
        serde_json::Deserializer::from_reader(buffer.as_slice()).into_iter::<serde_json::Value>();

    let meta = stream
        .next()
        .ok_or_else(|| ReflectionError::Generic("missing meta".into()))?
        .map_err(|err| ReflectionError::ErrorSerde(std::sync::Arc::new(err)))?;

    let values = stream
        .next()
        .ok_or_else(|| ReflectionError::Generic("missing values".into()))?
        .map_err(|err| lgn_data_model::ReflectionError::ErrorSerde(std::sync::Arc::new(err)))?;

    /*if let Some(object) = values.as_object_mut() {
        object.insert("meta".into(), meta);
    }*/

    let meta = serde_json::from_value::<crate::offline::Metadata>(meta)
        .map_err(|err| lgn_data_model::ReflectionError::ErrorSerde(std::sync::Arc::new(err)))?;
    let mut instance = meta.type_id.kind.new_instance();
    lgn_data_model::json_utils::reflection_apply_json_edit(instance.as_reflect_mut(), &values)?;
    *crate::get_meta_mut(instance.as_mut()) = meta;
    Ok(instance)
}

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
