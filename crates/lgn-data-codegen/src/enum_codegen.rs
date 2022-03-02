use std::collections::HashMap;

use crate::{
    enum_meta_info::{EnumMetaInfo, EnumVariantMetaInfo},
    GenerationType, ModuleMetaInfo,
};
use proc_macro2::TokenStream;
use quote::quote;

/// Generate token stream for enum variant definition
fn generate_enum_variants(variants: &[EnumVariantMetaInfo]) -> Vec<TokenStream> {
    variants
        .iter()
        .map(|m| {
            let member_ident = &m.name;
            if let Some(value) = &m.discriminant {
                quote! { #member_ident = #value, }
            } else {
                quote! { #member_ident, }
            }
        })
        .collect()
}

/// Generate token stream for a From<String> match arms
fn generate_from_string_match_arms(variants: &[EnumVariantMetaInfo]) -> Vec<TokenStream> {
    variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.name;
            let variant_name = &variant.name.to_string();
            quote! { #variant_name => Self::#variant_ident, }
        })
        .collect()
}

/// Generate the JSON write serialization for members.
/// Don't serialize members at default values
/// Skip 'transient' value
fn generate_field_descriptors(
    enum_meta_info: &EnumMetaInfo,
    variant_info: &EnumVariantMetaInfo,
    gen_type: Option<GenerationType>,
) -> Vec<TokenStream> {
    let enum_type_name = &enum_meta_info.name;

    variant_info
        .members
        .iter()
        .filter(|m| {
            (gen_type == Some(GenerationType::OfflineFormat) && !m.is_runtime_only())
                || (gen_type == Some(GenerationType::RuntimeFormat) && !m.is_offline_only())
        })
        .map(|m| {
            let variant_type_name = &variant_info.name;
            let member_ident = &m.name;
            let member_name = m.name.to_string();

            let member_type = if gen_type == Some(GenerationType::OfflineFormat) {
                m.type_path.clone()
            } else {
                m.get_runtime_type()
            };
            let attribute_impl = m.attributes.generate_descriptor_impl();

            quote! {
                lgn_data_model::FieldDescriptor {
                    field_name : #member_name.into(),
                    offset: memoffset::offset_of!(#enum_type_name::#variant_type_name, #member_ident),
                    field_type : <#member_type as lgn_data_model::TypeReflection>::get_type_def(),
                    attributes : #attribute_impl
                },
            }
        })
        .collect()
}

/*
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
*/

/// Generate token stream for variant descriptors
fn generate_enum_variant_descriptors(
    enum_meta_info: &EnumMetaInfo,
    gen_type: Option<GenerationType>,
) -> Vec<TokenStream> {
    enum_meta_info
        .variants
        .iter()
        .map(|variant_meta_info| {
            let variant_name = variant_meta_info.name.to_string();
            let attribute_impl = variant_meta_info.attributes.generate_descriptor_impl();
            let fields_descriptors =
                generate_field_descriptors(enum_meta_info, variant_meta_info, gen_type);
            quote! {
                lgn_data_model::EnumVariantDescriptor {
                    variant_name: #variant_name.into(),
                    attributes: #attribute_impl,
                    fields: vec![
                        #(#fields_descriptors)*
                    ],
                },
            }
        })
        .collect()
}

/// Generate token stream for custom serde implementation
/// Serialize as "String" in human readable or as u32 value in binary format
fn generate_serde_impls(enum_meta_info: &EnumMetaInfo) -> TokenStream {
    let variant_ser = enum_meta_info
        .variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.name;
            let variant_name = &variant.name.to_string();
            quote! { Self::#variant_ident => serializer.serialize_str(#variant_name), }
        })
        .collect::<Vec<TokenStream>>();

    let variant_deser = enum_meta_info
        .variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.name;
            let variant_name = &variant.name.to_string();
            quote! { #variant_name => Self::#variant_ident, }
        })
        .collect::<Vec<TokenStream>>();

    let enum_ident = &enum_meta_info.name;

    quote! {
        impl serde::Serialize for #enum_ident {
            #[allow(unsafe_code)]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                if serializer.is_human_readable() {
                    match self {
                        #(#variant_ser)*
                    }
                } else {
                    let u32_value = unsafe { *(self as *const Self).cast::<u32>() };
                    serializer.serialize_u32(u32_value)
                }
            }
        }

        impl<'de> serde::Deserialize<'de> for #enum_ident {
            #[allow(unsafe_code)]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let v = {
                    if deserializer.is_human_readable() {
                        let value = String::deserialize(deserializer)?;
                        match value.as_str() {
                            #(#variant_deser)*
                            _ => Self::default(),
                        }
                    } else {
                        let u32_value = u32::deserialize(deserializer)?;
                        unsafe { *(&u32_value as *const u32).cast::<Self>() }
                    }
                };
                Ok(v)
            }
        }
    }
}

pub(crate) fn generate_reflection(
    enum_meta_info: &EnumMetaInfo,
    gen_type: Option<GenerationType>,
) -> TokenStream {
    let enum_name = &enum_meta_info.name;

    // Grab the 'default_literal' or first option
    let default_value = if let Some(default_literal) = &enum_meta_info.attributes.default_literal {
        default_literal.clone()
    } else if let Some(variant) = enum_meta_info.variants.get(0) {
        let variant_name = &variant.name;
        quote! { Self::#variant_name}
    } else {
        quote! {}
    };

    let enum_variants = generate_enum_variants(&enum_meta_info.variants);
    let enum_from_string_match_arms = generate_from_string_match_arms(&enum_meta_info.variants);
    let enum_variants_descriptors = generate_enum_variant_descriptors(&enum_meta_info, gen_type);

    let serde_impls = generate_serde_impls(enum_meta_info);
    let enum_attributes = enum_meta_info.attributes.generate_descriptor_impl();
    let repr_impl = if enum_meta_info.has_only_unit_variants() {
        quote! { #[repr(u32)]}
    } else {
        quote! {}
    };

    quote! {
        #[derive(PartialEq, Clone, Copy)]
        #[non_exhaustive]
        #repr_impl
        pub enum #enum_name {
            #(#enum_variants)*
        }

        #serde_impls

        impl lgn_data_model::TypeReflection for #enum_name {
            fn get_type(&self) -> lgn_data_model::TypeDefinition { Self::get_type_def() }

            #[allow(unused_mut)]
            #[allow(clippy::let_and_return)]
            fn get_type_def() -> lgn_data_model::TypeDefinition {
                lgn_data_model::implement_enum_descriptor!(#enum_name,
                    #enum_attributes,
                    vec![
                    #(#enum_variants_descriptors)*
                    ]
                );
                lgn_data_model::TypeDefinition::Enum(&TYPE_DESCRIPTOR)
            }
            fn get_option_def() -> lgn_data_model::TypeDefinition {
                lgn_data_model::implement_option_descriptor!(#enum_name);
                lgn_data_model::TypeDefinition::Option(&OPTION_DESCRIPTOR)
            }
            fn get_array_def() -> lgn_data_model::TypeDefinition {
                lgn_data_model::implement_array_descriptor!(#enum_name);
                lgn_data_model::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
            }
        }

        impl From<String> for #enum_name {
            fn from(s: String)-> Self {
                return match s.as_str() {
                    #(#enum_from_string_match_arms)*
                    _ => Self::default()
                }
            }
        }

        impl Default for #enum_name {
            fn default() -> Self {
                #default_value
            }
        }
    }
}

pub(crate) fn generate_top_module_reflection(
    module_meta_infos: &HashMap<String, ModuleMetaInfo>,
) -> TokenStream {
    let enum_codegen: Vec<_> = module_meta_infos
        .iter()
        .flat_map(|(_mod_name, module_meta_info)| &module_meta_info.enum_meta_infos)
        .filter(|enum_meta_info| enum_meta_info.has_only_unit_variants())
        .map(|enum_meta_info| generate_reflection(enum_meta_info, None))
        .collect::<Vec<TokenStream>>();

    quote! {
        #(#enum_codegen)*
    }
}
