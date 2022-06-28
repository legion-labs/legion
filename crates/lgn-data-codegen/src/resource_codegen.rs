use std::collections::HashMap;

use crate::{struct_meta_info::StructMetaInfo, GenerationType, ModuleMetaInfo};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Generate Code for Resource Registration
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

    if !entries.is_empty() {
        let resource_registries = entries.iter().map(|type_name| {
            quote! {
            .add_resource_installer(
                <#type_name as lgn_data_runtime::ResourceDescriptor>::TYPE,
                lgn_data_offline::JsonInstaller::<#type_name>::create())
            }
        });

        let register_types = entries.iter().map(|type_name| {
            quote! {
                <#type_name as lgn_data_runtime::Resource>::register_resource_type();
            }
        });

        quote! {
            pub(crate) fn register_types(asset_registry: &mut lgn_data_runtime::AssetRegistryOptions) -> &mut lgn_data_runtime::AssetRegistryOptions {
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

    let offline_name =
        if resource_struct_info.only_generation == Some(GenerationType::OfflineFormat) {
            format!("{}", resource_struct_info.name).to_lowercase()
        } else {
            format!("offline_{}", resource_struct_info.name).to_lowercase()
        };

    let indexable_resource_crate = if resource_struct_info.parent_crate == "lgn_data_offline" {
        quote! { crate }
    } else {
        quote! { lgn_data_offline }
    };

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

            fn get_resource_type(&self) -> lgn_data_runtime::ResourceType {
                <#offline_identifier as lgn_data_runtime::ResourceDescriptor>::TYPE
            }
        }

        impl #indexable_resource_crate::SourceResource for #offline_identifier {
        }

    }
}
