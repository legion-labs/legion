use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::reflection::{DataContainerMetaInfo, MemberMetaInfo};
type QuoteRes = quote::__private::TokenStream;

/// Generate Runtime members definition
fn generate_runtime_fields(members: &[MemberMetaInfo]) -> Vec<QuoteRes> {
    members
        .iter()
        .filter(|m| !m.offline)
        .map(|m| {
            let member_ident = format_ident!("{}", &m.name);
            let runtime_type = m.get_runtime_type();

            quote! { pub #member_ident : #runtime_type, }
        })
        .collect()
}

/// Generate 'Default' implementation for runtime members
fn generate_runtime_defaults(members: &[MemberMetaInfo]) -> Vec<QuoteRes> {
    members
        .iter()
        .filter(|m| !m.offline)
        .map(|m| {
            let member_ident = format_ident!("{}", &m.name);
            if let Some(default_value) = &m.default_literal {
                if let Ok(syn::Lit::Str(_) | syn::Lit::ByteStr(_)) =
                    syn::parse2::<syn::Lit>(default_value.clone())
                {
                    quote! { #member_ident : #default_value.into(),}
                } else {
                    quote! { #member_ident : #default_value, }
                }
            } else if m.is_option() {
                quote! {#member_ident : None, }
            } else if m.is_vec() {
                quote! {#member_ident : Vec::new(), }
            } else {
                quote! { #member_ident : Default::default(), }
            }
        })
        .collect()
}

/// Generate code `AssetLoader` Runtime Registration
pub fn generate_registration_code(structs: &[DataContainerMetaInfo]) -> TokenStream {
    let entries: Vec<QuoteRes> = structs
        .iter()
        .map(|struct_meta| {
            let type_name = format_ident!("{}", &struct_meta.name);
            quote! { .add_loader::<#type_name>() }
        })
        .collect();
    quote! {
        pub fn add_loaders(registry: legion_data_runtime::AssetRegistryOptions) -> legion_data_runtime::AssetRegistryOptions {
            registry
            #(#entries)*
        }
    }
}

pub fn generate(data_container_info: &DataContainerMetaInfo, add_uses: bool) -> TokenStream {
    let runtime_identifier = format_ident!("{}", data_container_info.name);
    let runtime_name = format!("runtime_{}", data_container_info.name).to_lowercase();
    let runtime_loader = format_ident!("{}Loader", data_container_info.name);
    let runtime_fields = generate_runtime_fields(&data_container_info.members);
    let runtime_fields_defaults = generate_runtime_defaults(&data_container_info.members);

    let life_time = if data_container_info.need_life_time() {
        quote! {<'r>}
    } else {
        quote! {}
    };

    let use_quotes = if add_uses {
        quote! {
        use std::{any::Any, io};
        use serde::{Deserialize, Serialize};
        use legion_data_runtime::{Asset, AssetLoader,Resource,Reference};
        }
    } else {
        quote! {}
    };

    quote! {

        #use_quotes

        // Runtime Structure
        #[derive(Serialize, Deserialize)]
        pub struct #runtime_identifier #life_time {
            #(#runtime_fields)*
        }

        impl #life_time Resource for #runtime_identifier #life_time {
            const TYPENAME: &'static str = #runtime_name;
        }

        // Runtime default implementation
        impl #life_time Default for #runtime_identifier #life_time {
            fn default() -> Self {
                Self {
                    #(#runtime_fields_defaults)*
                }
            }
        }

        impl #life_time Asset for #runtime_identifier #life_time {
            type Loader = #runtime_loader;
        }

        #[derive(Default)]
        pub struct #runtime_loader {}

        impl AssetLoader for #runtime_loader {
            fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
                let output : #runtime_identifier = bincode::deserialize_from(reader).map_err(|_err|
                    io::Error::new(io::ErrorKind::InvalidData, "Failed to parse"))?;
                Ok(Box::new(output))
            }

            fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
        }
    }
}
