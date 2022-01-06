use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::reflection::DataContainerMetaInfo;

/// Generate Code for Resource Registration
pub fn generate_registration_code(structs: &[DataContainerMetaInfo]) -> TokenStream {
    let entries: Vec<TokenStream> = structs
        .iter()
        .filter(|struct_meta| struct_meta.is_resource)
        .map(|struct_meta| {
            let type_name = format_ident!("{}", &struct_meta.name);
            quote! { .add_type::<#type_name>() }
        })
        .collect();

    if !entries.is_empty() {
        quote! {
            pub fn register_resource_types(registry: lgn_data_offline::resource::ResourceRegistryOptions) -> lgn_data_offline::resource::ResourceRegistryOptions {
                registry
                #(#entries)*
            }
        }
    } else {
        quote! {}
    }
}

#[allow(clippy::too_many_lines)]
pub fn generate(data_container_info: &DataContainerMetaInfo, add_uses: bool) -> TokenStream {
    let offline_identifier = format_ident!("{}", data_container_info.name);
    let offline_name = format!("offline_{}", data_container_info.name).to_lowercase();
    let offline_identifier_processor = format_ident!("{}Processor", data_container_info.name);

    let use_quotes = if add_uses {
        let imports = data_container_info.imports();
        quote! {
            use std::{any::Any, io};
            use lgn_data_offline::{
                resource::{OfflineResource, ResourceProcessor},
            };
            use lgn_data_runtime::{Asset, AssetLoader, Resource};
            #(use #imports;)*
        }
    } else {
        quote! {}
    };

    quote! {

        #use_quotes

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
                let mut instance = #offline_identifier::default();
                let values : serde_json::Value = serde_json::from_reader(reader)
                    .map_err(|_err| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json"))?;
                lgn_data_model::json_utils::reflection_apply_json_edit::<#offline_identifier>(&mut instance, &values)
                    .map_err(|_err| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json"))?;
                Ok(Box::new(instance))
            }

            fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
        }


        impl ResourceProcessor for #offline_identifier_processor {
            fn new_resource(&mut self) -> Box<dyn Any + Send + Sync> {
                Box::new(#offline_identifier::default())
            }

            fn extract_build_dependencies(&mut self, _resource: &dyn Any) -> Vec<lgn_data_offline::ResourcePathId> {
                vec![]
            }

            fn get_resource_type_name(&self) -> Option<&'static str> {
                Some(#offline_identifier::TYPENAME)
            }

            fn write_resource(&mut self, resource: &dyn Any, writer: &mut dyn std::io::Write) -> std::io::Result<usize> {
                let instance = resource.downcast_ref::<#offline_identifier>().unwrap();
                let values = lgn_data_model::json_utils::reflection_save_relative_json(instance, #offline_identifier::get_default_instance()).
                    map_err(|_err| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json"))?;

                serde_json::to_writer_pretty(writer, &values).
                    map_err(|_err| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json"))?;
                Ok(1)
            }


            fn read_resource(&mut self,reader: &mut dyn std::io::Read) -> std::io::Result<Box<dyn Any + Send + Sync>> {
                self.load(reader)
            }

            fn get_resource_reflection<'a>(&self, resource: &'a dyn Any) -> Option<&'a dyn lgn_data_model::TypeReflection> {
                if let Some(instance) = resource.downcast_ref::<#offline_identifier>() {
                    return Some(instance);
                }
                None
            }

            fn get_resource_reflection_mut<'a>(&self, resource: &'a mut dyn Any) -> Option<&'a mut dyn lgn_data_model::TypeReflection> {
                if let Some(instance) = resource.downcast_mut::<#offline_identifier>() {
                    return Some(instance);
                }
                None
            }

        }
    }
}
