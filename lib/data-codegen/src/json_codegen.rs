use crate::reflection::{DataContainerMetaInfo, MemberMetaInfo};
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
                    self.#member_ident = serde_json::from_value(value.clone()).unwrap();
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
                if self.#member_ident != #default_ident.#member_ident {
                    if let Ok(json_string) = serde_json::to_string(&self.#member_ident) {
                        writer.write_all(b",\n")?;
                        writer.write_all(#prop_id)?;
                        writer.write_all(json_string.as_bytes())?;
                    }
                }
            }
        })
        .collect()
}

pub fn generate(data_container_info: &DataContainerMetaInfo) -> TokenStream {
    let offline_identifier = format_ident!("{}", data_container_info.name);
    let offline_default_instance =
        format_ident!("DEFAULT_{}", data_container_info.name.to_uppercase());

    let offline_name = &data_container_info.name;
    let offline_name_as_byte_string = Literal::byte_string(offline_name.as_bytes());
    let offline_fields_json_reads = generate_offline_json_reads(&data_container_info.members);
    let offline_fields_json_writes =
        generate_offline_json_writes(&offline_default_instance, &data_container_info.members);

    quote! {

        impl #offline_identifier {

            #[allow(clippy::missing_errors_doc)]
            pub fn read_from_json(&mut self, reader: &mut dyn std::io::Read) -> std::io::Result<()> {
                let values : serde_json::Value = serde_json::from_reader(reader).map_err(|_err| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json"))?;
                if values["_class"] == #offline_name {
                    #(#offline_fields_json_reads)*
                }
                else {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,"Invalid class identifier"));
                }
                Ok(())
            }

            #[allow(clippy::float_cmp, clippy::missing_errors_doc)]
            pub fn write_to_json(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
                writer.write_all(b"{\n\t\"_class\" : \"")?;
                writer.write_all(#offline_name_as_byte_string)?;
                writer.write_all(b"\"")?;
                #(#offline_fields_json_writes)*
                writer.write_all(b"\n}\n")?;
                Ok(())
            }
        }
    }
}
