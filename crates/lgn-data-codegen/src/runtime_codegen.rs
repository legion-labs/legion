use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{struct_meta_info::StructMetaInfo, GenerationType, ModuleMetaInfo};

/// Generate code `AssetLoader` Runtime Registration
pub(crate) fn generate_registration_code(
    module_meta_infos: &HashMap<String, ModuleMetaInfo>,
    gen_type: GenerationType,
) -> TokenStream {
    let entries: Vec<_> = module_meta_infos
        .iter()
        .flat_map(|(_mod_name, module_meta_info)| &module_meta_info.struct_meta_infos)
        .filter(|struct_meta| struct_meta.is_resource && !struct_meta.should_skip(gen_type))
        .map(|struct_meta| &struct_meta.name)
        .collect();

    let default_installers : Vec<_> = entries.iter().map(|type_name| {
            quote! {
                .add_default_resource_installer(<#type_name as lgn_data_runtime::ResourceDescriptor>::TYPE,
                    lgn_data_runtime::BincodeInstaller::<#type_name>::create())
            }
        })
        .collect();

    let register_types = entries.iter().map(|type_name| {
        quote! {
            <#type_name as lgn_data_runtime::Resource>::register_resource_type();
        }
    });

    if !entries.is_empty() {
        quote! {
            pub(crate) fn register_types(registry: &mut lgn_data_runtime::AssetRegistryOptions) -> &mut lgn_data_runtime::AssetRegistryOptions {
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

    let runtime_name = if struct_info.only_generation == Some(GenerationType::RuntimeFormat) {
        format!("{}", struct_info.name).to_lowercase()
    } else {
        format!("runtime_{}", struct_info.name).to_lowercase()
    };

    let runtime_reftype = format_ident!("{}ReferenceType", struct_info.name);

    quote! {

        impl lgn_data_runtime::ResourceDescriptor for #runtime_identifier {
            const TYPENAME: &'static str = #runtime_name;
        }

        #[async_trait::async_trait]
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
            fn get_resource_type(&self) -> lgn_data_runtime::ResourceType {
                <Self as lgn_data_runtime::ResourceDescriptor>::TYPE
            }

            async fn from_reader(reader: &mut lgn_data_runtime::AssetRegistryReader) -> Result<Box<Self>, lgn_data_runtime::AssetRegistryError> {
                lgn_data_runtime::from_binary_reader::<Self>(reader).await
            }
        }

        lgn_data_model::implement_reference_type_def!(#runtime_reftype, #runtime_identifier);


    }
}
