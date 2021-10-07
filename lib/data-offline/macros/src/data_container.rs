// ['DataContainer'] serialization

<<<<<<< HEAD
use proc_macro::{TokenStream, TokenTree};
use quote::*;
=======
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
>>>>>>> f7209cc0 (new lints in data-offline, data-compiler)
use syn::{parse_macro_input, DeriveInput};
type QuoteRes = quote::__private::TokenStream;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const LEGION_TAG: &str = "legion";
const DEFAULT_ATTR: &str = "default";
const HIDDEN_ATTR: &str = "hidden";
const OFFLINE_ATTR: &str = "offline";
const TOOLTIP_ATTR: &str = "tooltip";
const READONLY_ATTR: &str = "readonly";
const CATEGORY_ATTR: &str = "category";
const TRANSIENT_ATTR: &str = "transient";

fn metadata_from_type(t: &syn::Type) -> Option<QuoteRes> {
    match t {
        syn::Type::BareFn(fun) => Some(quote! {#fun}),
        syn::Type::Path(type_path) => Some(quote! {#type_path}),
        syn::Type::Reference(reference) => Some(quote! {#reference}),
        _ => None,
    }
}

<<<<<<< HEAD
=======
// Extract all `#[legion(...)]` attributes
fn get_legion_meta_items(attr: &syn::Attribute) -> Vec<syn::NestedMeta> {
    if attr.path != LEGION_TAG {
        return Vec::new();
    }

    match attr.parse_meta() {
        Ok(List(meta)) => meta.nested.into_iter().collect(),
        Ok(_other) => {
            panic!("Legion proc-macro: expected attributes syntax #[legion(...)");
        }
        Err(err) => {
            panic!("Legion proc-macro: Error parsing attributes, {}", err);
        }
    }
}

#[derive(Default)]
>>>>>>> f7209cc0 (new lints in data-offline, data-compiler)
struct MemberMetaInfo {
    name: String,
    type_id: syn::Type,
    type_name: String,
    offline: bool,
    category: Option<String>,
    hidden: bool,
    readonly: bool,
    transient: bool,
    tooltip: Option<String>,
    default_literal: Option<QuoteRes>,
}

impl MemberMetaInfo {
    fn calculate_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.name.hash(&mut hasher);
        self.type_name.hash(&mut hasher);
        hasher.finish()
    }

    fn is_option(&self) -> bool {
        if let syn::Type::Path(type_path) = self.type_id.clone() {
            !type_path.path.segments.is_empty() && type_path.path.segments[0].ident == "Option"
        } else {
            false
        }
    }

    fn get_runtime_type(&self) -> QuoteRes {
        let member_type = &self.type_id;
        if self.type_name.as_str() == "String" {
            quote! {&'r str }
        } else {
            quote! { #member_type }
        }
    }

    fn clone_on_compile(&self) -> bool {
        if let syn::Type::Path(type_path) = self.type_id.clone() {
            !type_path.path.segments.is_empty() && type_path.path.segments[0].ident == "Vec"
        } else {
            false
        }
    }
}

fn get_attribute_literal(
    group_iter: &mut std::iter::Peekable<proc_macro::token_stream::IntoIter>,
) -> Option<String> {
    if let Some(TokenTree::Punct(punct)) = group_iter.next() {
        if punct.as_char() == '=' {
            if let Some(TokenTree::Literal(lit)) = group_iter.next() {
                return Some(lit.to_string());
            }
        }
    }
    panic!("Legion proc-macro: invalid literal for attribute");
}

// Retrieive the token for the "default" attributes. Manually parse token to support tuple, arrays, constants, literal.
fn get_default_token_stream(
    group_iter: &mut std::iter::Peekable<proc_macro::token_stream::IntoIter>,
) -> Option<QuoteRes> {
    if let Some(TokenTree::Punct(punct)) = group_iter.next() {
        if punct.as_char() == '=' {
            if let Some(default_value) = group_iter.next() {
                let value = match default_value {
                    TokenTree::Ident(ident) => {
                        let mut token_ident = ident.to_string();
                        loop {
                            if let Some(TokenTree::Punct(punct)) = group_iter.peek() {
                                if punct.as_char() == ':' || punct.as_char() == '.' {
                                    token_ident.push(punct.as_char());
                                } else {
                                    break;
                                }
                            } else if let Some(TokenTree::Ident(ident)) = group_iter.peek() {
                                token_ident.push_str(ident.to_string().as_str());
                            } else if let Some(TokenTree::Group(group)) = group_iter.peek() {
                                token_ident.push_str(group.to_string().as_str());
                                group_iter.next();
                                break;
                            } else {
                                break;
                            }
                            group_iter.next();
                        }
                        let token_val: proc_macro2::TokenStream = token_ident.parse().unwrap();

                        Some(quote! { #token_val })
                    }

                    TokenTree::Literal(lit) => {
                        let lit_str = lit.to_string();
                        let token_val: proc_macro2::TokenStream = lit_str.parse().unwrap();
                        Some(quote! { #token_val })
                    }

                    TokenTree::Group(group) => {
                        let token_val: proc_macro2::TokenStream =
                            group.to_string().parse().unwrap();
                        Some(quote! { #token_val.into() })
                    }
                    TokenTree::Punct(_punct) => panic!(
                        "Legion proc-macro: unexpected punct in syntax for attribute 'default'"
                    ),
                };
                return value;
            }
        }
    }
    panic!("Legion proc-macro: invalid syntax for attribute 'default'");
}

fn get_member_info(field: &syn::Field) -> Option<MemberMetaInfo> {
    let field_type = metadata_from_type(&field.ty)?;

    let mut member_info = MemberMetaInfo {
        name: field.ident.as_ref().unwrap().to_string(),
        type_id: field.ty.clone(),
        type_name: format!("{}", field_type),
        category: None,
        offline: false,
        hidden: false,
        readonly: false,
        transient: false,
        tooltip: None,
        default_literal: None,
    };

    field
        .attrs
        .iter()
<<<<<<< HEAD
        .filter(|attr| attr.path.is_ident(LEGION_TAG))
        .for_each(|attr| {
            let token_stream = TokenStream::from(attr.tokens.clone());
            let mut token_iter = token_stream.into_iter();

            if let Some(TokenTree::Group(group)) = token_iter.next() {
                let mut group_iter = group.stream().into_iter().peekable();

                while let Some(TokenTree::Ident(ident)) = group_iter.next() {
                    match ident.to_string().as_str() {
                        DEFAULT_ATTR => {
                            member_info.default_literal = get_default_token_stream(&mut group_iter);
                        }
                        READONLY_ATTR => member_info.readonly = true,
                        HIDDEN_ATTR => member_info.hidden = true,
                        OFFLINE_ATTR => member_info.offline = true,
                        TRANSIENT_ATTR => member_info.transient = true,
                        TOOLTIP_ATTR => {
                            member_info.tooltip = get_attribute_literal(&mut group_iter);
                        }
                        CATEGORY_ATTR => {
                            member_info.category = get_attribute_literal(&mut group_iter);
                        }
                        _ => {}
                    }
=======
        .flat_map(|attr| get_legion_meta_items(attr))
    {
        match &meta_item {
            // Parse `#[legion(offline)]`
            Meta(Path(word)) if word == OFFLINE_ATTR => {
                member_info.offline = true;
            }
>>>>>>> f7209cc0 (new lints in data-offline, data-compiler)

                    if let Some(TokenTree::Punct(punct)) = group_iter.next() {
                        if punct.as_char() != ',' {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        });

    Some(member_info)
}

fn generate_runtime_fields(members: &[MemberMetaInfo]) -> Vec<QuoteRes> {
    members
        .iter()
        .filter(|m| !m.offline)
        .map(|m| {
            let member_ident = format_ident!("{}", &m.name);
            let runtime_type = m.get_runtime_type();

            quote! { #member_ident : #runtime_type, }
        })
        .collect()
}

fn generate_runtime_defaults(members: &[MemberMetaInfo]) -> Vec<QuoteRes> {
    members
        .iter()
        .filter(|m| !m.offline)
        .map(|m| {
            let member_ident = format_ident!("{}", &m.name);
            if let Some(default_value) = &m.default_literal {
                quote! { #member_ident : #default_value, }
            } else if m.is_option() {
                quote! {#member_ident : None, }
            } else {
                quote! { #member_ident : Default::default(), }
            }
        })
        .collect()
}

/// Generate code to convert Offline field to Runtime field
fn generate_runtime_from_offline(members: &[MemberMetaInfo]) -> Vec<QuoteRes> {
    members
        .iter()
        .filter(|m| !m.offline)
        .map(|m| {
            let member_ident = format_ident!("{}", &m.name);
            if m.type_name == "String" {
                quote! { #member_ident : self.#member_ident.as_str(), }
            } else if m.clone_on_compile() {
                quote! { #member_ident : self.#member_ident.clone(), }
            } else {
                quote! { #member_ident : self.#member_ident, }
            }
        })
        .collect()
}

/// Generate 'Default' implementation for offline members
fn generate_offline_defaults(members: &[MemberMetaInfo]) -> Vec<QuoteRes> {
    members
        .iter()
        .map(|m| {
            let member_ident = format_ident!("{}", &m.name);
            if let Some(default_value) = &m.default_literal {
                // If the default is a string literal, add "into()" to convert to String
                if let Ok(syn::Lit::Str(_) | syn::Lit::ByteStr(_)) =
                    syn::parse::<syn::Lit>(default_value.clone().into())
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
fn generate_offline_json_writes(members: &[MemberMetaInfo]) -> Vec<QuoteRes> {
    members
        .iter()
        .filter(|m| !m.transient)
        .map(|m| {
            let member_ident = format_ident!("{}", &m.name);
            let prop_id = format!("\t\"{}\" : ", &m.name);
            quote! {
                if self.#member_ident != default_obj.#member_ident {
                    if let Ok(json_string) = serde_json::to_string(&self.#member_ident) {
                        writer.write_all(",\n".as_bytes())?;
                        writer.write_all(#prop_id.as_bytes())?;
                        writer.write_all(&json_string.as_bytes())?;
                    }
                }
            }
        })
        .collect()
}

#[allow(clippy::too_many_lines)]
pub fn derive_data_container(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let offline_identifier = ast.ident.clone();
    let offline_name = format!("{}", ast.ident);
    let runtime_ident = format_ident!("Runtime{}", offline_identifier);
    let mut need_life_time = false;
    let mut members = Vec::new();

    // Hasher for the class signature
    // Class Name, Member Name, Member Type, Default Value Attribute
    let mut hasher = DefaultHasher::new();
    offline_identifier.hash(&mut hasher);

    match ast.data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(named_fields) => {
                for field in named_fields.named {
                    if let Some(member_info) = get_member_info(&field) {
                        hasher.write_u64(member_info.calculate_hash());
                        if member_info.type_name == "String" {
                            need_life_time = true;
                        }
                        members.push(member_info);
                    } else {
                        panic!(
                            "Legion: unsupported field type: {}",
                            field.ident.as_ref().unwrap().to_string(),
                        );
                    }
                }
            }
            syn::Fields::Unnamed(_) => panic!("only named fields are supported"),
            syn::Fields::Unit => panic!("unit fields not expected"),
        },
        syn::Data::Enum(_) => panic!("enums not supported"),
        syn::Data::Union(_) => panic!("unions not supported"),
    }

    // Optional lifetime parameters
    let life_time = if need_life_time {
        quote! {<'r>}
    } else {
        quote! {}
    };

    let offline_fields_defaults = generate_offline_defaults(&members);
    let offline_fields_json_reads = generate_offline_json_reads(&members);
    let offline_fields_json_writes = generate_offline_json_writes(&members);

    let runtime_fields = generate_runtime_fields(&members);
    let runtime_fields_defaults = generate_runtime_defaults(&members);
    let runtime_fields_from_offline = generate_runtime_from_offline(&members);

    let signature_hash = hasher.finish();

    TokenStream::from(quote! {

        // Runtime Structure
        #[derive(Debug, Serialize, Deserialize)]
        #[cfg(any(feature = "runtime_data", feature = "offline_data"))]
        pub struct #runtime_ident#life_time {
            #(#runtime_fields)*
        }

        // Runtime default implementation
        #[cfg(any(feature = "runtime_data", feature = "offline_data"))]
        impl#life_time Default for #runtime_ident#life_time {
            fn default() -> Self {
                Self {
                    #(#runtime_fields_defaults)*
                }
            }
        }

        // Offline default implementation
        #[cfg(feature = "offline_data")]
        impl Default for #offline_identifier {
            fn default() -> Self {
                Self {
                    #(#offline_fields_defaults)*
                }
            }
        }

        // Offline Json serialization
        #[cfg(feature = "offline_data")]
        impl OfflineDataContainer for #offline_identifier {

            fn read_from_json(&mut self, json_data: &str) -> std::io::Result<()> {
                let values: serde_json::Value = serde_json::from_str(json_data)
                .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json"))?;

                if values["_class"] == #offline_name {
                    #(#offline_fields_json_reads)*
                }
                else {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,"Invalid class identifier"));
                }
                Ok(())
            }

            #[allow(clippy::float_cmp)]
            fn write_to_json(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
                let default_obj = Self { ..Default::default() };
                writer.write_all("{\n\t\"_class\" : \"".as_bytes())?;
                writer.write_all(#offline_name.as_bytes())?;
                writer.write_all("\"".as_bytes())?;
                #(#offline_fields_json_writes)*
                writer.write_all("\n}\n".as_bytes())?;
                Ok(())
            }

            fn compile_runtime(&self) -> Result<Vec<u8>, String> {
                let runtime = #runtime_ident {
                    #(#runtime_fields_from_offline)*
                };
                let compiled_asset = bincode::serialize(&runtime).map_err(|err| "invalid serialization")?;
                Ok(compiled_asset)
            }

            const SIGNATURE_HASH : u64 = #signature_hash;
        }
    })
}
