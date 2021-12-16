use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::reflection::{DataContainerMetaInfo, MemberMetaInfo};
use crate::GenerationType;

/// Generate fields members definition
fn generate_fields(members: &[MemberMetaInfo], gen_type: GenerationType) -> Vec<TokenStream> {
    members
        .iter()
        .filter(|m| {
            gen_type == GenerationType::OfflineFormat
                || (gen_type == GenerationType::RuntimeFormat && !m.offline)
        })
        .map(|m| {
            let member_ident = format_ident!("{}", &m.name);
            let type_id = match gen_type {
                GenerationType::OfflineFormat => m.type_path.clone(),
                GenerationType::RuntimeFormat => m.get_runtime_type().unwrap(),
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
            gen_type == GenerationType::OfflineFormat
                || (gen_type == GenerationType::RuntimeFormat && !m.offline)
        })
        .map(|m| {
            let member_type = match gen_type {
                GenerationType::OfflineFormat => m.type_path.clone(),
                GenerationType::RuntimeFormat => m.get_runtime_type().unwrap(),
            };

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
    data_container_info: &DataContainerMetaInfo,
    gen_type: GenerationType,
) -> Vec<TokenStream> {
    data_container_info
        .members
        .iter()
        .filter(|m| {
            gen_type == GenerationType::OfflineFormat
                || (gen_type == GenerationType::RuntimeFormat && !m.offline)
        })
        .map(|m| {
            let struct_type_name = format_ident!("{}", &data_container_info.name);
            let member_ident = format_ident!("{}", &m.name);
            let prop_name = &m.name;
            let group = &m.group;

            let member_type = match gen_type {
                GenerationType::OfflineFormat => m.type_path.clone(),
                GenerationType::RuntimeFormat => m.get_runtime_type().unwrap(),
            };

            quote! {
                lgn_data_reflection::FieldDescriptor {
                    field_name : #prop_name.into(),
                    offset: memoffset::offset_of!(#struct_type_name, #member_ident),
                    field_type : <#member_type as lgn_data_reflection::TypeReflection>::get_type_def(),
                    group : #group.into()
                },
            }
        })
        .collect()
}

#[allow(clippy::too_many_lines)]
pub fn generate_reflection(
    data_container_info: &DataContainerMetaInfo,
    gen_type: GenerationType,
) -> TokenStream {
    let type_identifier = format_ident!("{}", data_container_info.name);
    let fields = generate_fields(&data_container_info.members, gen_type);
    let fields_defaults = generate_defaults(&data_container_info.members, gen_type);
    let default_instance = format_ident!("__{}_DEFAULT", data_container_info.name.to_uppercase());

    let signature_hash = data_container_info.calculate_hash();
    let fields_descriptors = generate_fields_descriptors(data_container_info, gen_type);

    quote! {
        #[derive(serde::Serialize, serde::Deserialize)]
        pub struct #type_identifier {
            #(#fields)*
        }

        impl #type_identifier {
            const SIGNATURE_HASH: u64 = #signature_hash;
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

        impl lgn_data_reflection::TypeReflection for #type_identifier {

            fn get_type(&self) -> lgn_data_reflection::TypeDefinition {
                    Self::get_type_def()
            }

            fn get_type_def() -> lgn_data_reflection::TypeDefinition {
                lgn_data_reflection::implement_struct_descriptor!(#type_identifier, vec![
                    #(#fields_descriptors)*
                ]);
                lgn_data_reflection::TypeDefinition::Struct(&TYPE_DESCRIPTOR)
            }
            fn get_option_def() -> lgn_data_reflection::TypeDefinition {
                lgn_data_reflection::implement_option_descriptor!(#type_identifier);
                lgn_data_reflection::TypeDefinition::Option(&OPTION_DESCRIPTOR)
            }
            fn get_array_def() -> lgn_data_reflection::TypeDefinition {
                lgn_data_reflection::implement_array_descriptor!(#type_identifier);
                lgn_data_reflection::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
            }
        }

        lazy_static::lazy_static! {
            static ref #default_instance: #type_identifier = #type_identifier {
                ..#type_identifier::default()
            };
        }
    }
}
