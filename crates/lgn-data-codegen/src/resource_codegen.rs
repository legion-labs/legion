use std::collections::HashMap;

use crate::{struct_meta_info::StructMetaInfo, ModuleMetaInfo};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Generate Code for Resource Registration
pub(crate) fn generate_registration_code(
    module_meta_infos: &HashMap<String, ModuleMetaInfo>,
) -> TokenStream {
    let entries: Vec<_> = module_meta_infos
        .iter()
        .flat_map(|(_mod_name, module_meta_info)| &module_meta_info.struct_meta_infos)
        .filter(|struct_meta| struct_meta.is_resource)
        .map(|struct_meta| &struct_meta.name)
        .collect();

    if !entries.is_empty() {
        let resource_registries = entries
            .iter()
            .map(|type_name| quote! { .add_processor_mut::<#type_name>() });

        let resources_loaders = entries
            .iter()
            .map(|type_name| quote! { .add_loader_mut::<#type_name>() });

        quote! {
            pub fn add_loaders(asset_registry: &mut lgn_data_runtime::AssetRegistryOptions) -> &mut lgn_data_runtime::AssetRegistryOptions {
                asset_registry
                #(#resources_loaders)*
                #(#resource_registries)*
            }
        }
    } else {
        quote! {}
    }
}

pub(crate) fn generate(resource_struct_info: &StructMetaInfo) -> TokenStream {
    let offline_identifier = format_ident!("{}", resource_struct_info.name);
    let offline_name = format!("offline_{}", resource_struct_info.name).to_lowercase();
    let offline_identifier_processor = format_ident!("{}Processor", resource_struct_info.name);

    quote! {

        impl lgn_data_runtime::ResourceDescriptor for #offline_identifier {
            const TYPENAME : &'static str = #offline_name;
        }

        impl lgn_data_runtime::Resource for #offline_identifier {
            fn as_reflect(&self) -> &dyn lgn_data_model::TypeReflection {
                self
            }
            fn as_reflect_mut(&mut self) -> &mut dyn lgn_data_model::TypeReflection {
                self
            }
            fn clone_dyn(&self) -> Box<dyn lgn_data_runtime::Resource> {
                Box::new(self.clone())
            }
            fn get_meta(&self) -> &lgn_data_runtime::Metadata {
                &self.meta
            }
            fn get_meta_mut(&mut self) -> &mut lgn_data_runtime::Metadata {
                &self.meta
            }
        }

        impl lgn_data_runtime::Asset for #offline_identifier {
            type Loader = #offline_identifier_processor;
        }

        impl lgn_data_runtime::OfflineResource for #offline_identifier {
            type Processor = #offline_identifier_processor;
        }

        #[derive(Default)]
        pub struct #offline_identifier_processor {}

        impl lgn_data_runtime::AssetLoader for #offline_identifier_processor {
            fn load(&mut self, reader: &mut dyn std::io::Read) -> Result<Box<dyn lgn_data_runtime::Resource>, lgn_data_runtime::AssetLoaderError> {
                let mut instance = #offline_identifier::default();

                let values : serde_json::Value = serde_json::from_reader(reader)
                    .map_err(|err| lgn_data_runtime::AssetLoaderError::ErrorLoading(<#offline_identifier as lgn_data_runtime::ResourceDescriptor>::TYPENAME,
                        format!("Error parsing json values ({})", err)))?;

                lgn_data_model::json_utils::reflection_apply_json_edit(&mut instance, &values)
                    .map_err(|err| lgn_data_runtime::AssetLoaderError::ErrorLoading(<#offline_identifier as lgn_data_runtime::ResourceDescriptor>::TYPENAME, err.to_string()))?;
                Ok(Box::new(instance))
            }

            fn load_init(&mut self, _asset: &mut (dyn lgn_data_runtime::Resource)) {}
        }


        impl lgn_data_runtime::ResourceProcessor for #offline_identifier_processor {
            fn new_resource(&mut self) -> Box<dyn lgn_data_runtime::Resource> {
                Box::new(#offline_identifier::default())
            }

            fn extract_build_dependencies(&mut self, resource: &dyn lgn_data_runtime::Resource) -> Vec<lgn_data_runtime::ResourcePathId> {
                let instance = resource.downcast_ref::<#offline_identifier>().unwrap();
                lgn_data_runtime::extract_resource_dependencies(instance)
                    .unwrap_or_default()
                    .into_iter()
                    .collect()
            }

            fn get_resource_type_name(&self) -> Option<&'static str> {
                Some(<#offline_identifier as lgn_data_runtime::ResourceDescriptor>::TYPENAME)
            }

            fn write_resource(&self, resource: &dyn lgn_data_runtime::Resource, writer: &mut dyn std::io::Write) -> Result<usize, lgn_data_runtime::ResourceProcessorError> {
                let instance = resource.downcast_ref::<#offline_identifier>().unwrap();
                let values = lgn_data_model::json_utils::reflection_save_relative_json(instance, #offline_identifier::get_default_instance())?;

                serde_json::to_writer_pretty(writer, &values).
                    map_err(|err| lgn_data_runtime::ResourceProcessorError::ResourceSerializationFailed(<#offline_identifier as lgn_data_runtime::ResourceDescriptor>::TYPENAME, err.to_string()))?;
                Ok(1)
            }


            fn read_resource(&mut self,reader: &mut dyn std::io::Read) -> Result<Box<dyn lgn_data_runtime::Resource>, lgn_data_runtime::ResourceProcessorError> {
                use lgn_data_runtime::AssetLoader;
                Ok(self.load(reader)?)
            }
        }
    }
}
