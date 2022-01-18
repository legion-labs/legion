use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::reflection::DataContainerMetaInfo;

/// Generate code `AssetLoader` Runtime Registration
pub fn generate_registration_code(structs: &[DataContainerMetaInfo]) -> TokenStream {
    let entries: Vec<TokenStream> = structs
        .iter()
        .filter(|struct_meta| struct_meta.is_resource)
        .map(|struct_meta| {
            let type_name = format_ident!("{}", &struct_meta.name);
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

pub fn generate(data_container_info: &DataContainerMetaInfo, add_uses: bool) -> TokenStream {
    let runtime_identifier = format_ident!("{}", data_container_info.name);
    let runtime_name = format!("runtime_{}", data_container_info.name).to_lowercase();
    let runtime_loader = format_ident!("{}Loader", data_container_info.name);
    let runtime_reftype = format_ident!("{}ReferenceType", data_container_info.name);

    let life_time = if data_container_info.need_life_time() {
        quote! {<'r>}
    } else {
        quote! {}
    };

    let use_quotes = if add_uses {
        let imports = data_container_info.runtime_imports();
        quote! {
            #(use #imports;)*
        }
    } else {
        quote! {}
    };

    quote! {

        #use_quotes

        impl #life_time lgn_data_runtime::Resource for #runtime_identifier #life_time {
            const TYPENAME: &'static str = #runtime_name;
        }

        impl #life_time lgn_data_runtime::Asset for #runtime_identifier #life_time {
            type Loader = #runtime_loader;
        }

        #[derive(serde::Serialize,serde::Deserialize,PartialEq)]
        pub struct #runtime_reftype (lgn_data_runtime::Reference<#runtime_identifier>);
        impl #runtime_reftype {
            pub fn id(&self) -> lgn_data_runtime::ResourceTypeAndId { self.0.id() }
        }

        lgn_data_model::implement_primitive_type_def!(#runtime_reftype);

        #[derive(Default)]
        pub struct #runtime_loader {}

        impl lgn_data_runtime::AssetLoader for #runtime_loader {
            fn load(&mut self, reader: &mut dyn std::io::Read) -> std::io::Result<Box<dyn std::any::Any + Send + Sync>> {
                let output : #runtime_identifier = bincode::deserialize_from(reader).map_err(|_err|
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to parse"))?;
                Ok(Box::new(output))
            }

            fn load_init(&mut self, _asset: &mut (dyn std::any::Any + Send + Sync)) {}
        }
    }
}
