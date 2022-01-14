use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

//type QuoteRes = quote::__private::TokenStream;
use lgn_utils::DefaultHasher;
use proc_macro2::{TokenStream, TokenTree};
use quote::{quote, ToTokens};

const LEGION_TAG: &str = "legion";
const DEFAULT_ATTR: &str = "default";
const HIDDEN_ATTR: &str = "hidden";
const OFFLINE_ATTR: &str = "offline";
const RUNTIME_ONLY_ATTR: &str = "runtime_only";
const IGNORE_DEPS_ATTR: &str = "ignore_deps";
const TOOLTIP_ATTR: &str = "tooltip";
const READONLY_ATTR: &str = "readonly";
const GROUP_ATTR: &str = "group";
const TRANSIENT_ATTR: &str = "transient";
const RESOURCE_TYPE_ATTR: &str = "resource_type";

pub struct DataContainerMetaInfo {
    pub name: String,
    pub need_life_time: bool,
    pub members: Vec<MemberMetaInfo>,
    pub is_resource: bool,
    pub is_component: bool,
}

#[allow(clippy::struct_excessive_bools)]
pub struct MemberMetaInfo {
    pub name: String,
    pub type_path: syn::Path,
    pub offline_imports: Vec<syn::Path>,
    pub runtime_imports: Vec<syn::Path>,
    pub default_literal: Option<TokenStream>,
    pub attributes: BTreeMap<String, String>,
}

impl DataContainerMetaInfo {
    #[allow(clippy::unused_self)]
    pub fn need_life_time(&self) -> bool {
        false
        // TODO: Add proper support for life_time with inplace deserialization
        //self.members.iter().any(|a| a.type_name == "String")
    }

    pub fn offline_imports(&self) -> Vec<syn::Path> {
        let mut output = vec![];
        for member in &self.members {
            for import in &member.offline_imports {
                if !output.contains(import) {
                    output.push(import.clone());
                }
            }
        }
        output
    }

    pub fn runtime_imports(&self) -> Vec<syn::Path> {
        let mut output = vec![];
        for member in &self.members {
            for import in &member.runtime_imports {
                if !output.contains(import) {
                    output.push(import.clone());
                }
            }
        }
        output
    }

    pub fn calculate_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.name.hash(&mut hasher);
        self.members.iter().for_each(|m| {
            m.name.hash(&mut hasher);
            m.get_type_name().hash(&mut hasher);

            m.attributes.iter().for_each(|(k, v)| {
                k.hash(&mut hasher);
                v.hash(&mut hasher);
            });

            m.default_literal
                .to_token_stream()
                .to_string()
                .hash(&mut hasher);
        });

        hasher.finish()
    }
}

pub fn get_data_container_info(
    item_struct: &syn::ItemStruct,
) -> Result<DataContainerMetaInfo, String> {
    let mut data_container_meta_info = DataContainerMetaInfo {
        name: item_struct.ident.to_string(),
        need_life_time: false,
        is_resource: item_struct
            .attrs
            .iter()
            .any(|attr| attr.path.segments.len() == 1 && attr.path.segments[0].ident == "resource"),
        is_component: item_struct.attrs.iter().any(|attr| {
            attr.path.segments.len() == 1 && attr.path.segments[0].ident == "component"
        }),
        members: Vec::new(),
    };

    if let syn::Fields::Named(named_fields) = &item_struct.fields {
        for field in &named_fields.named {
            if let syn::Type::Path(type_path) = &field.ty {
                data_container_meta_info
                    .members
                    .push(get_member_info(field, type_path.path.clone()));
            } else {
                let str = format!(
                    "Legion: unsupported field type: {}",
                    field.ident.as_ref().unwrap()
                );
                return Err(str);
            }
        }
    }
    Ok(data_container_meta_info)
}

impl MemberMetaInfo {
    pub fn is_option(&self) -> bool {
        !self.type_path.segments.is_empty() && self.type_path.segments[0].ident == "Option"
    }

    pub fn is_offline(&self) -> bool {
        self.attributes.contains_key(OFFLINE_ATTR)
    }

    pub fn is_runtime_only(&self) -> bool {
        self.attributes.contains_key(RUNTIME_ONLY_ATTR)
    }

    pub fn is_ingore_deps(&self) -> bool {
        self.attributes.contains_key(IGNORE_DEPS_ATTR)
    }

    pub fn is_vec(&self) -> bool {
        !self.type_path.segments.is_empty() && self.type_path.segments[0].ident == "Vec"
    }

    pub fn get_type_name(&self) -> String {
        self.type_path.to_token_stream().to_string()
    }

    pub fn get_runtime_type(&self) -> Option<syn::Path> {
        match self.get_type_name().as_str() {
            "Option < ResourcePathId >" => {
                let ty = if let Some(resource_type) = self.attributes.get(RESOURCE_TYPE_ATTR) {
                    format!("Option<{}ReferenceType>", resource_type)
                } else {
                    panic!("Option<ResourcePathId> must specify ResourceType in 'resource_type' attribute");
                };
                syn::parse_str(ty.as_str()).ok()
            }
            "Vec < ResourcePathId >" => {
                let ty = if let Some(resource_type) = self.attributes.get(RESOURCE_TYPE_ATTR) {
                    format!("Vec<{}ReferenceType>", resource_type)
                } else {
                    panic!("Vec<ResourcePathId> must specify ResourceType in 'resource_type' attribute");
                };
                syn::parse_str(ty.as_str()).ok()
            }
            _ => Some(self.type_path.clone()), // Keep same
        }
    }

    pub fn _clone_on_compile(&self) -> bool {
        !self.type_path.segments.is_empty() && self.type_path.segments[0].ident == "Vec"
    }
}

fn get_attribute_literal(
    group_iter: &mut std::iter::Peekable<proc_macro2::token_stream::IntoIter>,
) -> String {
    if let Some(TokenTree::Punct(punct)) = group_iter.next() {
        if punct.as_char() == '=' {
            if let Some(TokenTree::Literal(lit)) = group_iter.next() {
                return lit
                    .to_string()
                    .trim_start_matches('"')
                    .trim_end_matches('"')
                    .to_string();
            }
        }
    }
    panic!("Legion proc-macro: invalid literal for attribute");
}

// Retrieive the token for the "default" attributes. Manually parse token to
// support tuple, arrays, constants, literal.
fn get_default_token_stream(
    group_iter: &mut std::iter::Peekable<proc_macro2::token_stream::IntoIter>,
) -> Option<TokenStream> {
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

fn get_resource_type(
    group_iter: &mut std::iter::Peekable<proc_macro2::token_stream::IntoIter>,
) -> String {
    let mut attrib_str = String::new();

    if let Some(TokenTree::Punct(punct)) = group_iter.peek() {
        if punct.as_char() == '=' {
            group_iter.next();
        } else {
            panic!("Legion proc-macro: unexpected punct in syntax for attribute 'resource_type'")
        }
    }

    loop {
        match group_iter.peek() {
            Some(TokenTree::Punct(punct)) => {
                if punct.as_char() == ',' {
                    break;
                }
                attrib_str.push_str(&group_iter.next().unwrap().to_string());
            }
            None => break,
            Some(_) => attrib_str.push_str(&group_iter.next().unwrap().to_string()),
        }
    }

    if attrib_str.is_empty() {
        panic!("Legion proc-macro: empty  attribute 'resource_type'")
    }
    attrib_str
}

pub fn get_member_info(field: &syn::Field, type_path: syn::Path) -> MemberMetaInfo {
    let mut member_info = MemberMetaInfo {
        name: field.ident.as_ref().unwrap().to_string(),
        type_path,
        offline_imports: vec![],
        runtime_imports: vec![],
        attributes: BTreeMap::new(),
        default_literal: None,
    };

    field
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident(LEGION_TAG))
        .for_each(|attr| {
            let token_stream = attr.tokens.clone();
            let mut token_iter = token_stream.into_iter();

            if let Some(TokenTree::Group(group)) = token_iter.next() {
                let mut group_iter = group.stream().into_iter().peekable();

                while let Some(TokenTree::Ident(ident)) = group_iter.next() {
                    let ident = ident.to_string();
                    match ident.as_str() {
                        // Default literal token stream
                        DEFAULT_ATTR => {
                            member_info.default_literal = get_default_token_stream(&mut group_iter);
                        }

                        // Bool Attributes
                        READONLY_ATTR | HIDDEN_ATTR | OFFLINE_ATTR | TRANSIENT_ATTR
                        | RUNTIME_ONLY_ATTR | IGNORE_DEPS_ATTR => {
                            member_info.attributes.insert(ident, "true".into());
                        }

                        // ResourceType Attribute
                        RESOURCE_TYPE_ATTR => {
                            member_info
                                .attributes
                                .insert(ident, get_resource_type(&mut group_iter));
                            member_info
                                .offline_imports
                                .push(syn::parse_str("lgn_data_offline::ResourcePathId").unwrap());
                            member_info
                                .runtime_imports
                                .push(syn::parse_str("lgn_data_runtime::Reference").unwrap());
                        }
                        // Literal Attributes
                        GROUP_ATTR | TOOLTIP_ATTR => {
                            member_info
                                .attributes
                                .insert(ident, get_attribute_literal(&mut group_iter));
                        }
                        _ => {}
                    }

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

    member_info
}
