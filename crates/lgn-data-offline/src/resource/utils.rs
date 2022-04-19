/// Implement a new resourceType using raw processor
#[macro_export]
macro_rules! implement_raw_resource {
    ($type_id:ident, $processor:ident, $type_name:literal) => {
        #[derive(Default, Clone)]
        pub struct $type_id {
            pub content: Vec<u8>,
        }

        impl $type_id {
            pub fn register_type(asset_registry: &mut lgn_data_runtime::AssetRegistryOptions) {
                lgn_data_runtime::ResourceType::register_name(
                    <Self as lgn_data_runtime::ResourceDescriptor>::TYPE,
                    <Self as lgn_data_runtime::ResourceDescriptor>::TYPENAME,
                );
                let installer = std::sync::Arc::new($processor::default());
                asset_registry.add_resource_installer(
                    <Self as lgn_data_runtime::ResourceDescriptor>::TYPE,
                    installer.clone(),
                );
                asset_registry.add_processor(
                    <Self as lgn_data_runtime::ResourceDescriptor>::TYPE,
                    installer,
                );
            }
        }

        impl lgn_data_runtime::ResourceDescriptor for $type_id {
            const TYPENAME: &'static str = $type_name;
        }

        impl lgn_data_runtime::Resource for $type_id {
            fn as_reflect(&self) -> &dyn lgn_data_model::TypeReflection {
                self
            }
            fn as_reflect_mut(&mut self) -> &mut dyn lgn_data_model::TypeReflection {
                self
            }
            fn clone_dyn(&self) -> Box<dyn lgn_data_runtime::Resource> {
                Box::new(self.clone())
            }
        }

        impl lgn_data_model::TypeReflection for $type_id {
            fn get_type(&self) -> lgn_data_model::TypeDefinition {
                Self::get_type_def()
            }
            fn get_type_def() -> lgn_data_model::TypeDefinition {
                lgn_data_model::TypeDefinition::None
            }
        }

        #[derive(Default)]
        struct $processor {}

        #[async_trait::async_trait]
        impl lgn_data_runtime::ResourceInstaller for $processor {
            async fn install_from_stream(
                &self,
                resource_id: lgn_data_runtime::ResourceTypeAndId,
                request: &mut lgn_data_runtime::LoadRequest,
                reader: &mut lgn_data_runtime::AssetRegistryReader,
            ) -> Result<lgn_data_runtime::HandleUntyped, lgn_data_runtime::AssetRegistryError> {
                use tokio::io::AsyncReadExt;
                let mut content = Vec::new();
                reader.read_to_end(&mut content).await?;
                let handle = request
                    .asset_registry
                    .set_resource(resource_id, Box::new($type_id { content }))?;
                Ok(handle)
            }
        }

        impl lgn_data_runtime::ResourceProcessor for $processor {
            fn new_resource(&self) -> Box<dyn lgn_data_runtime::Resource> {
                Box::new($type_id::default())
            }

            fn extract_build_dependencies(
                &self,
                _resource: &dyn lgn_data_runtime::Resource,
            ) -> Vec<lgn_data_runtime::ResourcePathId> {
                vec![]
            }

            fn write_resource(
                &self,
                resource: &dyn lgn_data_runtime::Resource,
                writer: &mut dyn std::io::Write,
            ) -> Result<usize, lgn_data_runtime::AssetRegistryError> {
                if let Some(png) = resource.downcast_ref::<$type_id>() {
                    Ok(writer.write(png.content.as_slice()).map_err(|err| {
                        lgn_data_runtime::AssetRegistryError::ResourceSerializationFailed(
                            <$type_id as lgn_data_runtime::ResourceDescriptor>::TYPENAME,
                            err.to_string(),
                        )
                    })?)
                } else {
                    Err(
                        lgn_data_runtime::AssetRegistryError::ResourceSerializationFailed(
                            <$type_id as lgn_data_runtime::ResourceDescriptor>::TYPENAME,
                            "invalid cast".into(),
                        ),
                    )
                }
            }
        }
    };
}
