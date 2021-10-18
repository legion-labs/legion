use crate::reflection::DataContainerMetaInfo;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn generate(data_container_info: &DataContainerMetaInfo) -> TokenStream {
    let offline_identifier = format_ident!("{}", data_container_info.name);
    let offline_name = format!("offline_{}", data_container_info.name).to_lowercase();
    let offline_identifier_processor = format_ident!("{}Processor", data_container_info.name);

    quote! {

        use std::{any::Any, io};
        use legion_data_runtime::{Asset, AssetLoader, Resource};
        use legion_data_offline::{
            resource::{OfflineResource, ResourceProcessor},
        };

        impl Resource for #offline_identifier {
            const TYPENAME: &'static str = #offline_name;
        }

        impl Asset for #offline_identifier {
            type Loader = #offline_identifier_processor;
        }

        impl OfflineResource for #offline_identifier {
            type Processor = #offline_identifier_processor;
        }

        #[derive(Default)]
        pub struct #offline_identifier_processor {}

        impl AssetLoader for #offline_identifier_processor {
            fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
                let mut new_instance = #offline_identifier { ..Default::default()};
                new_instance.read_from_json(reader)?;
                Ok(Box::new(new_instance))
            }

            fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
        }

        impl ResourceProcessor for #offline_identifier_processor {
            fn new_resource(&mut self) -> Box<dyn Any + Send + Sync> {
                Box::new(#offline_identifier { ..Default::default() })
            }

            fn extract_build_dependencies(&mut self, _resource: &dyn Any) -> Vec<legion_data_offline::ResourcePathId> {
                vec![]
            }

            fn write_resource(&mut self, resource: &dyn Any, writer: &mut dyn std::io::Write) -> std::io::Result<usize> {
                let instance = resource.downcast_ref::<#offline_identifier>().unwrap();
                instance.write_to_json(writer)?;
                Ok(1)
            }

            fn read_resource(&mut self,reader: &mut dyn std::io::Read) -> std::io::Result<Box<dyn Any + Send + Sync>> {
                self.load(reader)
            }
        }
    }
}
