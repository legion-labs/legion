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
        let resource_registries = entries.iter().map(|type_name| {
            let offline_identifier_processor = format_ident!("{}Processor", type_name);
            quote! {
            .add_processor(<#type_name as lgn_data_runtime::ResourceDescriptor>::TYPE,
                std::sync::Arc::new(#offline_identifier_processor::default()))
            .add_resource_installer(
                <#type_name as lgn_data_runtime::ResourceDescriptor>::TYPE,
                std::sync::Arc::new(#offline_identifier_processor::default()))
            }
        });

        let register_types = entries.iter().map(|type_name| {
            quote! {
                lgn_data_runtime::ResourceType::register_name(<#type_name as lgn_data_runtime::ResourceDescriptor>::TYPE, <#type_name as lgn_data_runtime::ResourceDescriptor>::TYPENAME);
            }
        });

        quote! {
            pub fn register_types(asset_registry: &mut lgn_data_runtime::AssetRegistryOptions) -> &mut lgn_data_runtime::AssetRegistryOptions {
                #(#register_types)*

                asset_registry
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
        }

        impl #offline_identifier {

            /// # Errors
            /// return a `AssetRegistryError` if it failed to create a resource from an async reader
            pub async fn from_json_reader(reader: &mut lgn_data_runtime::AssetRegistryReader) -> Result<Self, lgn_data_runtime::AssetRegistryError> {
                use tokio::io::AsyncReadExt;
                let mut instance = Self::default();
                let mut buffer = Vec::<u8>::new();
                reader.read_to_end(&mut buffer).await?;
                let values : serde_json::Value = serde_json::from_slice(buffer.as_slice())
                    .map_err(|err| lgn_data_model::ReflectionError::ErrorSerde(std::sync::Arc::new(err)))?;

                lgn_data_model::json_utils::reflection_apply_json_edit(&mut instance, &values)?;

                Ok(instance)
            }
        }

        #[derive(Default)]
        pub struct #offline_identifier_processor {}

        #[async_trait::async_trait]
        impl lgn_data_runtime::ResourceInstaller for #offline_identifier_processor {
            async fn install_from_stream(
                &self,
                resource_id: lgn_data_runtime::ResourceTypeAndId,
                request: &mut lgn_data_runtime::LoadRequest,
                reader: &mut lgn_data_runtime::AssetRegistryReader,
            ) -> Result<lgn_data_runtime::HandleUntyped, lgn_data_runtime::AssetRegistryError> {

                let instance = #offline_identifier::from_json_reader(reader).await?;
                let handle = request.asset_registry.set_resource(resource_id, Box::new(instance))?;
                Ok(handle)
            }
        }


        impl lgn_data_runtime::ResourceProcessor for #offline_identifier_processor {
            fn new_resource(&self) -> Box<dyn lgn_data_runtime::Resource> {
                Box::new(#offline_identifier::default())
            }

            fn extract_build_dependencies(&self, resource: &dyn lgn_data_runtime::Resource) -> Vec<lgn_data_runtime::ResourcePathId> {
                let instance = resource.downcast_ref::<#offline_identifier>().unwrap();
                lgn_data_runtime::extract_resource_dependencies(instance)
                    .unwrap_or_default()
                    .into_iter()
                    .collect()
            }

            fn write_resource(&self, resource: &dyn lgn_data_runtime::Resource, writer: &mut dyn std::io::Write) -> Result<usize, lgn_data_runtime::AssetRegistryError> {
                let instance = resource.downcast_ref::<#offline_identifier>().unwrap();
                let values = lgn_data_model::json_utils::reflection_save_relative_json(instance, #offline_identifier::get_default_instance())?;

                serde_json::to_writer_pretty(writer, &values).
                    map_err(|err| lgn_data_runtime::AssetRegistryError::ResourceSerializationFailed(<#offline_identifier as lgn_data_runtime::ResourceDescriptor>::TYPENAME, err.to_string()))?;
                Ok(1)
            }
        }
    }
}
