/// Implement a new resourceType using raw processor
#[macro_export]
macro_rules! implement_raw_resource {
    ($type_id:ident, $processor:ident, $type_name:literal) => {
        #[derive(Default, Clone)]
        pub struct $type_id {
            pub content: Vec<u8>,
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

        impl lgn_data_runtime::Asset for $type_id {
            type Loader = $processor;
        }
        impl lgn_data_runtime::OfflineResource for $type_id {
            type Processor = $processor;
        }

        #[derive(Default)]
        pub struct $processor {}

        impl lgn_data_runtime::AssetLoader for $processor {
            fn load(
                &mut self,
                reader: &mut dyn std::io::Read,
            ) -> Result<Box<dyn lgn_data_runtime::Resource>, lgn_data_runtime::AssetLoaderError>
            {
                let mut content = Vec::new();
                reader.read_to_end(&mut content)?;
                Ok(Box::new($type_id { content }))
            }
            fn load_init(&mut self, _asset: &mut (dyn lgn_data_runtime::Resource)) {}
        }

        impl lgn_data_runtime::ResourceProcessor for $processor {
            fn new_resource(&mut self) -> Box<dyn lgn_data_runtime::Resource> {
                Box::new($type_id::default())
            }

            fn extract_build_dependencies(
                &mut self,
                _resource: &dyn lgn_data_runtime::Resource,
            ) -> Vec<lgn_data_runtime::ResourcePathId> {
                vec![]
            }

            fn write_resource(
                &self,
                resource: &dyn lgn_data_runtime::Resource,
                writer: &mut dyn std::io::Write,
            ) -> Result<usize, lgn_data_runtime::ResourceProcessorError> {
                if let Some(png) = resource.downcast_ref::<$type_id>() {
                    Ok(writer.write(png.content.as_slice()).map_err(|err| {
                        lgn_data_runtime::ResourceProcessorError::ResourceSerializationFailed(
                            <$type_id as lgn_data_runtime::ResourceDescriptor>::TYPENAME,
                            err.to_string(),
                        )
                    })?)
                } else {
                    Err(
                        lgn_data_runtime::ResourceProcessorError::ResourceSerializationFailed(
                            <$type_id as lgn_data_runtime::ResourceDescriptor>::TYPENAME,
                            "invalid cast".into(),
                        ),
                    )
                }
            }

            fn read_resource(
                &mut self,
                reader: &mut dyn std::io::Read,
            ) -> Result<Box<dyn lgn_data_runtime::Resource>, lgn_data_runtime::ResourceProcessorError> {
                use lgn_data_runtime::AssetLoader;
                Ok(self.load(reader)?)
            }

            fn get_resource_type_name(&self) -> Option<&'static str> {
                Some($type_name)
            }
        }
    };
}
