use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{member_meta_info::MemberMetaInfo, struct_meta_info::StructMetaInfo, GenerationType};

/// Generate fields members definition
fn generate_fields(members: &[MemberMetaInfo], gen_type: GenerationType) -> Vec<TokenStream> {
    members
        .iter()
        .filter(|m| {
            (gen_type == GenerationType::OfflineFormat && !m.is_runtime_only())
                || (gen_type == GenerationType::RuntimeFormat && !m.is_offline_only())
        })
        .map(|m| {
            let member_ident = format_ident!("{}", &m.name);
            let type_id = match gen_type {
                GenerationType::OfflineFormat => m.type_path.clone(),
                GenerationType::RuntimeFormat => m.get_runtime_type(),
            };
            quote! { pub #member_ident : #type_id, }
        })
        .collect()
}

/// Generate 'Default' implementation for offline members
fn generate_defaults(members: &[MemberMetaInfo], gen_type: GenerationType) -> Vec<TokenStream> {
    members
        .iter()
        .filter(|m| {
            (gen_type == GenerationType::OfflineFormat && !m.is_runtime_only())
                || (gen_type == GenerationType::RuntimeFormat && !m.is_offline_only())
        })
        .map(|m| {
            let member_type = match gen_type {
                GenerationType::OfflineFormat => m.type_path.clone(),
                GenerationType::RuntimeFormat => m.get_runtime_type(),
            };
            let member_ident = format_ident!("{}", &m.name);
            if let Some(default_value) = &m.attributes.default_literal {
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
            } else if m.is_vec() {
                quote! { #member_ident : Vec::new(), }
            } else {
                quote! { #member_ident : #member_type::default(), }
            }
        })
        .collect()
}

/// Generate the JSON write serialization for members.
/// Don't serialize members at default values
/// Skip 'transient' value
fn generate_fields_descriptors(
    struct_info: &StructMetaInfo,
    gen_type: GenerationType,
) -> Vec<TokenStream> {
    struct_info
        .members
        .iter()
        .filter(|m| {
            (gen_type == GenerationType::OfflineFormat && !m.is_runtime_only())
                || (gen_type == GenerationType::RuntimeFormat && !m.is_offline_only())
        })
        .map(|m| {
            let struct_type_name = &struct_info.name;
            let member_ident = &m.name;
            let member_name = m.name.to_string();

            let member_type = match gen_type {
                GenerationType::OfflineFormat => m.type_path.clone(),
                GenerationType::RuntimeFormat => m.get_runtime_type(),
            };
            let attribute_impl = m.attributes.generate_descriptor_impl();

            quote! {
                lgn_data_model::FieldDescriptor {
                    field_name : #member_name.into(),
                    offset: memoffset::offset_of!(#struct_type_name, #member_ident),
                    field_type : <#member_type as lgn_data_model::TypeReflection>::get_type_def(),
                    attributes : #attribute_impl
                },
            }
        })
        .collect()
}

pub(crate) fn generate_reflection(
    struct_meta_info: &StructMetaInfo,
    gen_type: GenerationType,
) -> TokenStream {
    let type_identifier = &struct_meta_info.name;
    let fields = generate_fields(&struct_meta_info.members, gen_type);
    let fields_defaults = generate_defaults(&struct_meta_info.members, gen_type);
    let default_instance = format_ident!(
        "__{}_DEFAULT",
        struct_meta_info.name.to_string().to_uppercase()
    );

    let signature_hash = struct_meta_info.calculate_hash();
    let fields_descriptors = generate_fields_descriptors(struct_meta_info, gen_type);

    let dynamic_derive =
        if struct_meta_info.is_component && gen_type == GenerationType::RuntimeFormat {
            quote! { #[derive(lgn_ecs::component::Component, Clone)] }
        } else {
            quote! {}
        };

    let attribute_impl = struct_meta_info.attributes.generate_descriptor_impl();

    quote! {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq)]
        #dynamic_derive
        pub struct #type_identifier {
            #(#fields)*
        }

        impl #type_identifier {
            #[allow(dead_code)]
            const SIGNATURE_HASH: u64 = #signature_hash;
            #[allow(dead_code)]
            pub fn get_default_instance() -> &'static Self {
                &#default_instance
            }
        }

        #[allow(clippy::derivable_impls)]
        impl Default for #type_identifier {
            fn default() -> Self {
                Self {
                    #(#fields_defaults)*
                }
            }
        }

        impl lgn_data_model::TypeReflection for #type_identifier {

            fn get_type(&self) -> lgn_data_model::TypeDefinition {
                    Self::get_type_def()
            }

            #[allow(unused_mut)]
            #[allow(clippy::let_and_return)]
            fn get_type_def() -> lgn_data_model::TypeDefinition {
                lgn_data_model::implement_struct_descriptor!(#type_identifier,
                    #attribute_impl,
                    vec![
                    #(#fields_descriptors)*
                ]);
                lgn_data_model::TypeDefinition::Struct(&TYPE_DESCRIPTOR)
            }
            fn get_option_def() -> lgn_data_model::TypeDefinition {
                lgn_data_model::implement_option_descriptor!(#type_identifier);
                lgn_data_model::TypeDefinition::Option(&OPTION_DESCRIPTOR)
            }
            fn get_array_def() -> lgn_data_model::TypeDefinition {
                lgn_data_model::implement_array_descriptor!(#type_identifier);
                lgn_data_model::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
            }
        }

        lazy_static::lazy_static! {
            #[allow(clippy::needless_update)]
            static ref #default_instance: #type_identifier = #type_identifier::default();
        }
    }
}
