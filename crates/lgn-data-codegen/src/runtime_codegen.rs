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
        .map(|struct_meta| {
            let type_name = &struct_meta.name;
            quote! { .add_loader_mut::<#type_name>() }
        })
        .collect();

    if !entries.is_empty() {
        quote! {
            pub fn add_loaders(registry: &mut lgn_data_runtime::AssetRegistryOptions) -> &mut lgn_data_runtime::AssetRegistryOptions {
                registry
                #(#entries)*
            }
        }
    } else {
        quote! {}
    }
}

pub(crate) fn generate(struct_info: &StructMetaInfo) -> TokenStream {
    let runtime_identifier = &struct_info.name;
    let runtime_name = format!("runtime_{}", struct_info.name).to_lowercase();
    let runtime_loader = format_ident!("{}Loader", struct_info.name);
    let runtime_reftype = format_ident!("{}ReferenceType", struct_info.name);

    let life_time = if struct_info.need_life_time() {
        quote! {<'r>}
    } else {
        quote! {}
    };

    quote! {

        impl #life_time lgn_data_runtime::Resource for #runtime_identifier #life_time {
            const TYPENAME: &'static str = #runtime_name;
        }

        impl #life_time lgn_data_runtime::Asset for #runtime_identifier #life_time {
            type Loader = #runtime_loader;
        }

        lgn_data_model::implement_reference_type_def!(#runtime_reftype, #runtime_identifier);

        #[derive(Default)]
        pub struct #runtime_loader {}

        impl lgn_data_runtime::AssetLoader for #runtime_loader {
            fn load(&mut self, reader: &mut dyn std::io::Read) -> Result<Box<dyn std::any::Any + Send + Sync>, lgn_data_runtime::AssetLoaderError> {
                let output : #runtime_identifier = bincode::deserialize_from(reader)
                    .map_err(|err| lgn_data_runtime::AssetLoaderError::ErrorLoading(<#runtime_identifier as lgn_data_runtime::Resource>::TYPENAME, err.to_string()))?;

                Ok(Box::new(output))
            }

            fn load_init(&mut self, _asset: &mut (dyn std::any::Any + Send + Sync)) {}
        }
    }
}
