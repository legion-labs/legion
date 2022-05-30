/// Implement a new resourceType using raw processor
#[macro_export]
macro_rules! implement_raw_resource {
    ($type_id:ident, $processor:ident, $type_name:literal) => {
        use serde::{Deserialize, Serialize};

        #[derive(Clone, Serialize, Deserialize)]
        pub struct $type_id {
            pub meta: lgn_data_runtime::Metadata,

            #[serde(with = "serde_bytes")]
            pub raw_data: Vec<u8>,
        }

        impl Default for $type_id {
            fn default() -> Self {
                Self {
                    meta: lgn_data_runtime::Metadata::new(lgn_data_runtime::ResourcePathName::default(), <$type_id as lgn_data_runtime::ResourceDescriptor>::TYPENAME, <$type_id as lgn_data_runtime::ResourceDescriptor>::TYPE),
                    raw_data: vec![],
                }
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
            fn get_meta(&self) -> Option<&lgn_data_runtime::Metadata> {
                Some(&self.meta)
            }
            fn get_meta_mut(&mut self) -> Option<&mut lgn_data_runtime::Metadata> {
                Some(&mut self.meta)
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
                let resource: $type_id = serde_json::from_reader(reader).unwrap();
                Ok(Box::new(resource))
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
                let resource = resource.downcast_ref::<$type_id>().unwrap();
                serde_json::to_writer_pretty(writer, resource).unwrap();
                Ok(1) // no bytes written exposed by serde.
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

        impl lgn_data_offline::resource::RawContent for $type_id {
            fn set_raw_content(&mut self, data: &[u8]) {
                self.raw_data = data.to_vec();
            }
        }
    };
}
