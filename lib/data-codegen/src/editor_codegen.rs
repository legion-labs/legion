use crate::reflection::{DataContainerMetaInfo, MemberMetaInfo};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
type QuoteRes = quote::__private::TokenStream;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Generate the JSON write serialization for members.
/// Don't serialize members at default values
/// Skip 'transient' value
fn generate_offline_parse_str(members: &[MemberMetaInfo]) -> Vec<QuoteRes> {
    members
        .iter()
        .filter(|m| !m.transient)
        .map(|m| {
            let mut hasher = DefaultHasher::new();
            m.name.hash(&mut hasher);
            let hash_value: u64 = hasher.finish();
            let member_ident = format_ident!("{}", &m.name);
            quote! { #hash_value => self.#member_ident.parse_from_str(field_value)?, }
        })
        .collect()
}

/// Generate the Editor Property Descriptor info
fn generate_offline_editor_descriptors(
    default_ident: &syn::Ident,
    members: &[MemberMetaInfo],
) -> Vec<QuoteRes> {
    members
        .iter()
        .filter(|m| !m.transient)
        .map(|m| {
            let member_ident = format_ident!("{}", &m.name);
            let prop_name = &m.name;
            let group_name = &m.category;
            let prop_type = &m.type_name;
            quote! {
                PropertyDescriptor {
                    name : #prop_name,
                    type_name : #prop_type,
                    default_value : bincode::serialize(&#default_ident.#member_ident).map_err(|_err| "bincode error")?,
                    value : bincode::serialize(&self.#member_ident).map_err(|_err| "bincode error")?,
                    group : #group_name.into(),
                },
            }
        })
        .collect()
}

pub fn generate(data_container_info: &DataContainerMetaInfo) -> TokenStream {
    let offline_identifier = format_ident!("{}", data_container_info.name);
    let offline_default_instance =
        format_ident!("DEFAULT_{}", data_container_info.name.to_uppercase());

    let offline_fields_editor_descriptors = generate_offline_editor_descriptors(
        &offline_default_instance,
        &data_container_info.members,
    );

    let offline_fields_parse_str = generate_offline_parse_str(&data_container_info.members);

    quote! {


        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hasher,Hash};
        use legion_data_offline::data_container::{PropertyDescriptor,ParseFromStr};
        impl #offline_identifier {

            pub fn write_field_by_name(&mut self, field_name : &str, field_value : &str) -> Result<(), &'static str> {
                let mut hasher = DefaultHasher::new();
                field_name.hash(&mut hasher);
                match hasher.finish() {
                    #(#offline_fields_parse_str)*
                    _ => return Err("invalid field"),
                }
                Ok(())
            }

            pub fn get_editor_properties(&self) -> Result<Vec<legion_data_offline::data_container::PropertyDescriptor>, &'static str> {
                let output : Vec::<PropertyDescriptor> = vec![
                    #(#offline_fields_editor_descriptors)*
                ];
                Ok(output)
            }
        }
    }
}
