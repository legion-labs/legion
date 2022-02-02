use std::collections::HashMap;

use crate::{
    enum_meta_info::{EnumMetaInfo, EnumVariantMetaInfo},
    ModuleMetaInfo,
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

/// Generate token stream for variant descriptors
fn generate_enum_variant_descriptors(variants: &[EnumVariantMetaInfo]) -> Vec<TokenStream> {
    variants
        .iter()
        .map(|v| {
            let variant_name = v.name.to_string();
            let attribute_impl = v.attributes.generate_descriptor_impl();
            quote! {
                lgn_data_model::EnumVariantDescriptor {
                    variant_name : #variant_name.into(),
                    attributes : #attribute_impl
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
    module_meta_infos: &HashMap<String, ModuleMetaInfo>,
) -> TokenStream {
    let enum_codegen: Vec<_> = module_meta_infos
        .iter()
        .flat_map(|(_mod_name, module_meta_info)| &module_meta_info.enum_meta_infos)
        .map(|enum_meta_info| {
            let enum_name = &enum_meta_info.name;

            // Grab the 'default_literal' or first option
            let default_value =
                if let Some(default_literal) = &enum_meta_info.attributes.default_literal {
                    default_literal.clone()
                } else if let Some(variant) = enum_meta_info.variants.get(0) {
                    let variant_name = &variant.name;
                    quote! { #enum_name::#variant_name}
                } else {
                    quote! {}
                };

            let enum_variants = generate_enum_variants(&enum_meta_info.variants);
            let enum_from_string_match_arms =
                generate_from_string_match_arms(&enum_meta_info.variants);
            let enum_variants_descriptors =
                generate_enum_variant_descriptors(&enum_meta_info.variants);

            let serde_impls = generate_serde_impls(enum_meta_info);
            let enum_attributes = enum_meta_info.attributes.generate_descriptor_impl();

            quote! {
                #[derive(PartialEq, Clone, Copy)]
                #[non_exhaustive]
                #[repr(u32)]
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
        })
        .collect::<Vec<TokenStream>>();

    quote! {
        #(#enum_codegen)*
    }
}
