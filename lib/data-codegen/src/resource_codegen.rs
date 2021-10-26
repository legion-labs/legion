use crate::reflection::{DataContainerMetaInfo, MemberMetaInfo};
use legion_utils::DefaultHash;
use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};
type QuoteRes = quote::__private::TokenStream;

/// Generic the JSON read serialization.
/// Values not present in JSON will be initialized at default value
/// Skip 'transient' value
fn generate_offline_json_reads(members: &[MemberMetaInfo]) -> Vec<QuoteRes> {
    members
        .iter()
        .filter(|m| !m.transient)
        .map(|m| {
            let member_ident = format_ident!("{}", &m.name);
            let member_name = &m.name;
            quote! {
                if let Some(value) = values.get(#member_name) {
                    instance.#member_ident = serde_json::from_value(value.clone()).unwrap();
                }
            }
        })
        .collect()
}

/// Generate the JSON write serialization for members.
/// Don't serialize members at default values
/// Skip 'transient' value
fn generate_offline_json_writes(
    default_ident: &syn::Ident,
    members: &[MemberMetaInfo],
) -> Vec<QuoteRes> {
    members
        .iter()
        .filter(|m| !m.transient)
        .map(|m| {
            let member_ident = format_ident!("{}", &m.name);
            let prop_id = Literal::byte_string(format!("\t\"{}\" : ", &m.name).as_bytes());
            quote! {
                if instance.#member_ident != #default_ident.#member_ident {
                    if let Ok(json_string) = serde_json::to_string(&instance.#member_ident) {
                        writer.write_all(b",\n")?;
                        writer.write_all(#prop_id)?;
                        writer.write_all(json_string.as_bytes())?;
                    }
                }
            }
        })
        .collect()
}

/// Generate the JSON write serialization for members.
/// Don't serialize members at default values
/// Skip 'transient' value
fn generate_reflection_write(members: &[MemberMetaInfo]) -> Vec<QuoteRes> {
    members
        .iter()
        .filter(|m| !m.transient)
        .map(|m| {
            let hash_value = m.name.default_hash();
            let member_ident = format_ident!("{}", &m.name);
            quote! { #hash_value => {
                self.#member_ident = serde_json::from_str(field_value).map_err(|_err| "json serialization error")?;
            },
            }
        })
        .collect()
}

/// Generate the JSON write serialization for members.
/// Don't serialize members at default values
/// Skip 'transient' value
fn generate_reflection_read(
    default_ident: Option<&syn::Ident>,
    members: &[MemberMetaInfo],
) -> Vec<QuoteRes> {
    members
        .iter()
        .filter(|m| !m.transient)
        .map(|m| {
            let hash_value: u64 = m.name.default_hash();
            let member_ident = format_ident!("{}", &m.name);
            if let Some(default_ident) = default_ident {
                quote! { #hash_value => serde_json::to_string(&#default_ident.#member_ident).map_err(|_err| "json serialization error"),
                }
            }
            else {
                quote! { #hash_value => serde_json::to_string(&self.#member_ident).map_err(|_err| "json serialization error"),
                }
            }
        })
        .collect()
}

/// Generate the JSON write serialization for members.
/// Don't serialize members at default values
/// Skip 'transient' value
fn generate_property_descriptors(members: &[MemberMetaInfo]) -> Vec<QuoteRes> {
    members
        .iter()
        .map(|m| {
            let hash_value: u64 = m.name.default_hash();
            let prop_name = &m.name;
            let group_name = &m.category;
            let prop_type = &m.type_name;
            quote! {
                map.insert(#hash_value, PropertyDescriptor {
                    name : #prop_name,
                    type_name : #prop_type,
                    group : #group_name,
                });
            }
        })
        .collect()
}

#[allow(clippy::too_many_lines)]
pub fn generate(data_container_info: &DataContainerMetaInfo) -> TokenStream {
    let offline_identifier = format_ident!("{}", data_container_info.name);
    let offline_name = format!("offline_{}", data_container_info.name).to_lowercase();
    let offline_identifier_processor = format_ident!("{}Processor", data_container_info.name);

    //let offline_fields_parse_str = generate_offline_parse_str(&data_container_info.members);
    let offline_default_instance =
        format_ident!("DEFAULT_{}", data_container_info.name.to_uppercase());

    let offline_default_descriptor =
        format_ident!("__{}_DESCRIPTORS", data_container_info.name.to_uppercase());

    let reflection_writer = generate_reflection_write(&data_container_info.members);
    let reflection_read_default = generate_reflection_read(
        Some(&offline_default_instance),
        &data_container_info.members,
    );
    let reflection_read = generate_reflection_read(None, &data_container_info.members);

    let offline_fields_editor_descriptors =
        generate_property_descriptors(&data_container_info.members);

    let class_name = &data_container_info.name;
    let offline_fields_json_reads = generate_offline_json_reads(&data_container_info.members);
    let offline_fields_json_writes =
        generate_offline_json_writes(&offline_default_instance, &data_container_info.members);

    quote! {

        use std::{any::Any, io};
        use legion_data_offline::{PropertyDescriptor,
            resource::{OfflineResource, ResourceProcessor, ResourceReflection},
        };
        use legion_data_runtime::{Asset, AssetLoader, Resource};
        use legion_utils::DefaultHash;

        lazy_static::lazy_static! {
            static ref #offline_default_descriptor : HashMap<u64, PropertyDescriptor> = {
                let mut map = HashMap::new();
                #(#offline_fields_editor_descriptors)*
                map
            };
        }

        impl Resource for #offline_identifier {
            const TYPENAME: &'static str = #offline_name;
        }

        impl Asset for #offline_identifier {
            type Loader = #offline_identifier_processor;
        }

        impl OfflineResource for #offline_identifier {
            type Processor = #offline_identifier_processor;
        }

        #[derive(Default)]
        pub struct #offline_identifier_processor {}

        impl AssetLoader for #offline_identifier_processor {
            fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
                let mut instance = #offline_identifier { ..#offline_identifier::default()};

                let values : serde_json::Value = serde_json::from_reader(reader).map_err(|_err| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json"))?;
                if values["_class"] == #class_name {
                    #(#offline_fields_json_reads)*
                }
                else {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,"Invalid class identifier"));
                }
                Ok(Box::new(instance))
            }

            fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
        }

        impl ResourceReflection for #offline_identifier {
            /// Interface defining field serialization by name
            fn write_property(&mut self, field_name: &str, field_value: &str) -> Result<(), &'static str> {
                let mut hasher = DefaultHasher::new();
                field_name.hash(&mut hasher);
                match hasher.finish() {
                    #(#reflection_writer)*
                    _ => return Err("invalid field"),
                }
                Ok(())
            }

            /// Interface defining field serialization by name
            fn read_property(&self, field_name: &str) -> Result<String, &'static str> {
                let mut hasher = DefaultHasher::new();
                field_name.hash(&mut hasher);
                match hasher.finish() {
                    #(#reflection_read)*
                    _ => Err("invalid field"),
                }
            }

            /// Interface defining field serialization by name
            fn read_property_default(&self, field_name: &str) -> Result<String, &'static str> {
                let mut hasher = DefaultHasher::new();
                field_name.hash(&mut hasher);
                match hasher.finish() {
                    #(#reflection_read_default)*
                    _ => Err("invalid field"),
                }
            }

            fn get_property_descriptors(&self) -> Option<&'static HashMap<u64,PropertyDescriptor>> {
                Some(&#offline_default_descriptor)
            }
        }

        impl ResourceProcessor for #offline_identifier_processor {
            fn new_resource(&mut self) -> Box<dyn Any + Send + Sync> {
                Box::new(#offline_identifier { ..#offline_identifier::default() })
            }

            fn extract_build_dependencies(&mut self, _resource: &dyn Any) -> Vec<legion_data_offline::ResourcePathId> {
                vec![]
            }

            #[allow(clippy::float_cmp,clippy::too_many_lines)]
            fn write_resource(&mut self, resource: &dyn Any, writer: &mut dyn std::io::Write) -> std::io::Result<usize> {
                let instance = resource.downcast_ref::<#offline_identifier>().unwrap();
                writer.write_all(b"{\n\t\"_class\" : \"")?;
                writer.write_all(#class_name.to_string().as_bytes())?;
                writer.write_all(b"\"")?;
                #(#offline_fields_json_writes)*
                writer.write_all(b"\n}\n")?;
                Ok(1)
            }


            fn read_resource(&mut self,reader: &mut dyn std::io::Read) -> std::io::Result<Box<dyn Any + Send + Sync>> {
                self.load(reader)
            }

            fn get_resource_reflection<'a>(&self, resource: &'a dyn Any) -> Option<&'a dyn ResourceReflection> {
                if let Some(instance) = resource.downcast_ref::<#offline_identifier>() {
                    return Some(instance);
                }
                None
            }

            fn get_resource_reflection_mut<'a>(&self, resource: &'a mut dyn Any) -> Option<&'a mut dyn ResourceReflection> {
                if let Some(instance) = resource.downcast_mut::<#offline_identifier>() {
                    return Some(instance);
                }
                None
            }

        }
    }
}
