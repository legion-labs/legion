use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::reflection::{DataContainerMetaInfo, MemberMetaInfo};
//type QuoteRes = quote::__private::TokenStream;

/// Generate Offline members definition
fn generate_offline_fields(members: &[MemberMetaInfo]) -> Vec<TokenStream> {
    members
        .iter()
        .map(|m| {
            let member_ident = format_ident!("{}", &m.name);
            let offline_type = &m.type_id;
            quote! { pub #member_ident : #offline_type, }
        })
        .collect()
}

/// Generate 'Default' implementation for offline members
fn generate_offline_defaults(members: &[MemberMetaInfo]) -> Vec<TokenStream> {
    members
        .iter()
        .map(|m| {
            let member_ident = format_ident!("{}", &m.name);
            if let Some(default_value) = &m.default_literal {
                // If the default is a string literal, add "into()" to convert to String
                if let Ok(syn::Lit::Str(_) | syn::Lit::ByteStr(_)) =
                    syn::parse2::<syn::Lit>(default_value.clone())
                {
                    quote! { #member_ident : #default_value.into(),}
                } else {
                    quote! { #member_ident : #default_value, }
                }
            } else if m.is_option() {
                quote! { #member_ident : None, }
            } else {
                quote! { #member_ident : Default::default(), }
            }
        })
        .collect()
}

pub fn generate(data_container_info: &DataContainerMetaInfo) -> TokenStream {
    let offline_identifier = format_ident!("{}", data_container_info.name);
    let offline_fields = generate_offline_fields(&data_container_info.members);
    let offline_fields_defaults = generate_offline_defaults(&data_container_info.members);
    let offline_default_instance =
        format_ident!("DEFAULT_{}", data_container_info.name.to_uppercase());

    quote! {

        use serde::{Deserialize, Serialize};

        // Offline Structure
        #[derive(Debug, Serialize, Deserialize)]
        pub struct #offline_identifier {
            #(#offline_fields)*
        }

        // Offline default implementation
        impl Default for #offline_identifier {
            fn default() -> Self {
                Self {
                    #(#offline_fields_defaults)*
                }
            }
        }

        lazy_static::lazy_static! {
            static ref #offline_default_instance: #offline_identifier = #offline_identifier {
                ..#offline_identifier::default()
            };
        }
    }
}
