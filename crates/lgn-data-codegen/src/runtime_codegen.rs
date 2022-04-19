use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{struct_meta_info::StructMetaInfo, ModuleMetaInfo};

/// Generate code `AssetLoader` Runtime Registration
pub(crate) fn generate_registration_code(
    module_meta_infos: &HashMap<String, ModuleMetaInfo>,
) -> TokenStream {

    let entries: Vec<_> = module_meta_infos
        .iter()
        .flat_map(|(_mod_name, module_meta_info)| &module_meta_info.struct_meta_infos)
        .filter(|struct_meta| struct_meta.is_resource)
        .map(|struct_meta| &struct_meta.name).collect();
            
    let default_installers : Vec<_> = entries.iter().map(|type_name| {
            let default_installer = format_ident!("{}ResourceInstaller", type_name);
            quote! {
                .add_default_resource_installer(<#type_name as lgn_data_runtime::ResourceDescriptor>::TYPE, 
                    std::sync::Arc::new(#default_installer::default()))
            }
        })
        .collect();

    let register_types = entries.iter().map(|type_name| {
            quote! {
                lgn_data_runtime::ResourceType::register_name(<#type_name as lgn_data_runtime::ResourceDescriptor>::TYPE, <#type_name as lgn_data_runtime::ResourceDescriptor>::TYPENAME);
            }
        });

    if !entries.is_empty() {
        quote! {
            pub fn register_types(registry: &mut lgn_data_runtime::AssetRegistryOptions) -> &mut lgn_data_runtime::AssetRegistryOptions {
                #(#register_types)*

                registry
                #(#default_installers)*
            }
        }
    } else {
        quote! {}
    }
}

pub(crate) fn generate(struct_info: &StructMetaInfo) -> TokenStream {
    let runtime_identifier = &struct_info.name;
    let runtime_name = format!("runtime_{}", struct_info.name).to_lowercase();
    let runtime_installer = format_ident!("{}ResourceInstaller", struct_info.name);
    let runtime_reftype = format_ident!("{}ReferenceType", struct_info.name);

    quote! {

        impl lgn_data_runtime::ResourceDescriptor for #runtime_identifier {
            const TYPENAME: &'static str = #runtime_name;
        }

        impl lgn_data_runtime::Resource for #runtime_identifier {
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

        lgn_data_model::implement_reference_type_def!(#runtime_reftype, #runtime_identifier);

        impl #runtime_identifier {
            /// # Errors
            /// return a `AssetRegistryError` if it failed to create a resource from an async reader
            pub async fn from_reader(reader: &mut lgn_data_runtime::AssetRegistryReader) -> Result<Self, lgn_data_runtime::AssetRegistryError> {
                use tokio::io::AsyncReadExt;
                use lgn_data_runtime::ResourceDescriptor;
                let mut buffer = vec![];
                reader.read_to_end(&mut buffer).await?;
                let new_resource: Self = bincode::deserialize_from(&mut buffer.as_slice()).map_err(|err| {
                    lgn_data_runtime::AssetRegistryError::ResourceSerializationFailed(Self::TYPENAME, err.to_string())
                })?;
                Ok(new_resource)
            }
        }

        #[derive(Default)]
        pub struct #runtime_installer {}

        #[async_trait::async_trait]
        impl lgn_data_runtime::ResourceInstaller for #runtime_installer {
            async fn install_from_stream(
                &self,
                resource_id: lgn_data_runtime::ResourceTypeAndId,
                request: &mut lgn_data_runtime::LoadRequest,
                reader: &mut lgn_data_runtime::AssetRegistryReader,
            ) -> Result<lgn_data_runtime::HandleUntyped, lgn_data_runtime::AssetRegistryError> {

                let mut output = #runtime_identifier::from_reader(reader).await?;
                lgn_data_runtime::activate_reference(
                    resource_id,
                    &mut output,
                    request.asset_registry.clone(),
                )
                .await;
                let handle = request.asset_registry.set_resource(resource_id, Box::new(output))?;
                Ok(handle)
            }
        }
    }
}
