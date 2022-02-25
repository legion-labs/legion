/// Implement a new resourceType using raw processor
#[macro_export]
macro_rules! implement_raw_resource {
    ($type_id:ident, $processor:ident, $type_name:literal) => {

        #[derive(Default)]
        pub struct $type_id {
            pub content: Vec<u8>,
        }

        impl lgn_data_runtime::Resource for $type_id {
            const TYPENAME: &'static str = $type_name;
        }
        impl lgn_data_runtime::Asset for $type_id {
            type Loader = $processor;
        }
        impl lgn_data_offline::resource::OfflineResource for $type_id {
            type Processor = $processor;
        }

        #[derive(Default)]
        pub struct $processor {}

        impl lgn_data_runtime::AssetLoader for $processor {
            fn load(
                &mut self,
                reader: &mut dyn std::io::Read,
            ) -> Result<Box<dyn std::any::Any + Send + Sync>, lgn_data_runtime::AssetLoaderError> {
                let mut content = Vec::new();
                reader.read_to_end(&mut content)?;
                Ok(Box::new($type_id { content }))
            }
            fn load_init(&mut self, _asset: &mut (dyn std::any::Any + Send + Sync)) {}
        }

        impl lgn_data_offline::resource::ResourceProcessor for $processor {
            fn new_resource(&mut self) -> Box<dyn std::any::Any + Send + Sync> {
                Box::new($type_id::default())
            }

            fn extract_build_dependencies(
                &mut self,
                _resource: &dyn std::any::Any,
            ) -> Vec<lgn_data_offline::ResourcePathId> {
                vec![]
            }

            fn write_resource(
                &self,
                resource: &dyn std::any::Any,
                writer: &mut dyn std::io::Write,
            ) -> Result<usize, lgn_data_offline::resource::ResourceProcessorError> {
                if let Some(png) = resource.downcast_ref::<$type_id>() {
                    Ok(writer.write(png.content.as_slice())
                    .map_err(|err| lgn_data_offline::resource::ResourceProcessorError::ResourceSerializationFailed(<$type_id as lgn_data_runtime::Resource>::TYPENAME, err.to_string()))?)
                }
                else {
                    Err(lgn_data_offline::resource::ResourceProcessorError::ResourceSerializationFailed(<$type_id as lgn_data_runtime::Resource>::TYPENAME, "invalid cast".into()))
                }
            }

            fn read_resource(
                &mut self,
                reader: &mut dyn std::io::Read,
            ) -> Result<Box<dyn std::any::Any + Send + Sync>, lgn_data_offline::resource::ResourceProcessorError> {
                use lgn_data_runtime::AssetLoader;
                Ok(self.load(reader)?)
            }

            fn get_resource_type_name(&self) -> Option<&'static str> {
                Some($type_name)
            }
        }
    };
}
