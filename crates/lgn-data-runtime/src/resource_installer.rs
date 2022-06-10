use std::{str::FromStr, sync::Arc};

use async_trait::async_trait;
use futures::FutureExt;
use lgn_data_model::{utils::find_property_mut, TypeDefinition, TypeReflection};
use tokio::task::JoinHandle;

use crate::{
    from_binary_reader, AssetRegistry, AssetRegistryError, AssetRegistryReader, HandleUntyped,
    LoadRequest, ReferenceUntyped, Resource, ResourceTypeAndId,
};

/// Implement a default `ResourceInstaller` using `from_binary_reader` interface
pub struct BincodeInstaller<T: Resource> {
    _phantom: std::marker::PhantomData<T>,
}

impl<'de, T: Resource + Default + serde::Deserialize<'de>> BincodeInstaller<T> {
    /// Create a new Bincode installer for a specific Resource type
    pub fn create() -> Arc<dyn ResourceInstaller> {
        Arc::new(Self {
            _phantom: std::marker::PhantomData::<T>,
        })
    }
}

#[async_trait]
impl<'de, T: Resource + Default + serde::Deserialize<'de>> ResourceInstaller
    for BincodeInstaller<T>
{
    async fn install_from_stream(
        &self,
        resource_id: ResourceTypeAndId,
        request: &mut LoadRequest,
        reader: &mut AssetRegistryReader,
    ) -> Result<HandleUntyped, AssetRegistryError> {
        let mut new_resource: Box<dyn crate::Resource> = from_binary_reader::<T>(reader).await?;
        let resource = new_resource.as_mut();
        activate_reference(resource_id, resource, request.asset_registry.clone()).await;
        let handle = request
            .asset_registry
            .set_resource(resource_id, new_resource)?;
        Ok(handle)
    }
}

/// Trait to implement a `Resource` Installer
#[async_trait]
pub trait ResourceInstaller: Send + Sync {
    /// Install a resource from a stream
    async fn install_from_stream(
        &self,
        _resource_id: ResourceTypeAndId,
        _request: &mut LoadRequest,
        _reader: &mut AssetRegistryReader,
    ) -> Result<HandleUntyped, AssetRegistryError>;
}

type DependentLoadJob = Vec<(
    ResourceTypeAndId,
    String,
    JoinHandle<Result<HandleUntyped, AssetRegistryError>>,
)>;

/// Activate all the Reference field using reflection
pub async fn activate_reference<'a>(
    resource_id: ResourceTypeAndId,
    object: &mut dyn Resource,
    asset_registry: Arc<AssetRegistry>,
) {
    let mut results = Vec::new();
    internal_activate_reference(
        (object.as_reflect_mut() as *mut dyn TypeReflection).cast::<()>(),
        object.get_type(),
        String::new(),
        asset_registry,
        &mut results,
    );

    if !results.is_empty() {
        for (_depedent_id, path, job_result) in results {
            match job_result.await {
                Ok(load_result) => match load_result {
                    Ok(handle) => {
                        if let Ok(reflect_ptr) = find_property_mut(object.as_reflect_mut(), &path) {
                            if reflect_ptr
                                .base_descriptor
                                .type_name
                                .ends_with("ReferenceType")
                                && reflect_ptr.base_descriptor.size
                                    == std::mem::size_of::<ReferenceUntyped>()
                            {
                                lgn_tracing::debug!(
                                    "{:?}.{} activated ({:?}",
                                    resource_id,
                                    path,
                                    &handle
                                );
                                let reference =
                                    unsafe { &mut *(reflect_ptr.base.cast::<ReferenceUntyped>()) };
                                reference.activate(handle);
                            }
                        }
                    }
                    Err(load_err) => {
                        lgn_tracing::error!("{} (from {:?}{})", load_err, resource_id, path);
                    }
                },
                Err(job_error) => lgn_tracing::error!("{}", job_error),
            }
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
fn internal_activate_reference(
    base: *mut (),
    type_def: TypeDefinition,
    path: String,
    asset_registry: Arc<AssetRegistry>,
    results: &mut DependentLoadJob,
) {
    match type_def {
        TypeDefinition::None => {}
        TypeDefinition::BoxDyn(box_dyn_descriptor) => {
            let sub_type = (box_dyn_descriptor.get_inner_type)(base);
            let sub_base = (box_dyn_descriptor.get_inner_mut)(base);
            internal_activate_reference(sub_base, sub_type, path, asset_registry, results);
        }
        TypeDefinition::Array(array_descriptor) => {
            for index in 0..(array_descriptor.len)(base) {
                if let Ok(base) = (array_descriptor.get_mut)(base, index) {
                    let new_path =
                        if let TypeDefinition::BoxDyn(box_dyn) = array_descriptor.inner_type {
                            format!("{}[{}]", path, {
                                (box_dyn.get_inner_type)(base).get_type_name()
                            })
                        } else {
                            format!("{}[{}]", path, index)
                        };

                    internal_activate_reference(
                        base,
                        array_descriptor.inner_type,
                        new_path,
                        asset_registry.clone(),
                        results,
                    );
                }
            }
        }
        TypeDefinition::Primitive(primitive_descriptor) => {
            if primitive_descriptor
                .base_descriptor
                .type_name
                .ends_with("ReferenceType")
            {
                let mut buffer = Vec::new();
                let mut json = serde_json::Serializer::new(&mut buffer);
                let mut serializer = <dyn erased_serde::Serializer>::erase(&mut json);
                match unsafe {
                    (primitive_descriptor.base_descriptor.dynamic_serialize)(base, &mut serializer)
                } {
                    Ok(()) => {
                        //let value = serde_json::Value::from_str(&buffer);
                        let value = String::from_utf8_lossy(&buffer);
                        let value = value.trim_start_matches('"').trim_end_matches('"');

                        if let Ok(resource_id) = ResourceTypeAndId::from_str(value) {
                            let handle: JoinHandle<Result<HandleUntyped, AssetRegistryError>> =
                                tokio::spawn(async move {
                                    asset_registry.load_async_untyped(resource_id).await
                                }.boxed());
                            results.push((resource_id, path, handle));
                        }
                    }
                    Err(err) => {
                        lgn_tracing::error!("Failed to activate reference {}", err);
                    }
                }
            }
        }
        TypeDefinition::Enum(_enum_descriptor) => {}
        TypeDefinition::Option(option_descriptor) => {
            if let Some(value_base) = unsafe { (option_descriptor.get_inner_mut)(base) } {
                internal_activate_reference(
                    value_base,
                    option_descriptor.inner_type,
                    format!("{}.0", path),
                    asset_registry,
                    results,
                );
            }
        }
        TypeDefinition::Struct(struct_descriptor) => {
            struct_descriptor.fields.iter().for_each(|field| {
                let ignore = field
                    .attributes
                    .as_ref()
                    .and_then(|a| a.get("ignore_deps"))
                    .is_some();

                if !ignore {
                    let field_base = unsafe { base.cast::<u8>().add(field.offset).cast::<()>() };

                    let new_path = if path.is_empty() {
                        field.field_name.clone()
                    } else {
                        format!("{}.{}", path, field.field_name)
                    };

                    internal_activate_reference(
                        field_base,
                        field.field_type,
                        new_path,
                        asset_registry.clone(),
                        results,
                    );
                }
            });
        }
    }
}
