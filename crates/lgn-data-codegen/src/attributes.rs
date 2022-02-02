use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use std::collections::BTreeMap;

pub(crate) const LEGION_TAG: &str = "legion";
pub(crate) const DEFAULT_ATTR: &str = "default";
pub(crate) const HIDDEN_ATTR: &str = "hidden";
pub(crate) const OFFLINE_ONLY_ATTR: &str = "offline_only";
pub(crate) const RUNTIME_ONLY_ATTR: &str = "runtime_only";
pub(crate) const IGNORE_DEPS_ATTR: &str = "ignore_deps";
pub(crate) const TOOLTIP_ATTR: &str = "tooltip";
pub(crate) const READONLY_ATTR: &str = "readonly";
pub(crate) const GROUP_ATTR: &str = "group";
pub(crate) const TRANSIENT_ATTR: &str = "transient";
pub(crate) const EDITOR_TYPE_ATTR: &str = "editor_type";
pub(crate) const RESOURCE_TYPE_ATTR: &str = "resource_type";

/// Parsed #legion attributes on a type (Struct, Field, Enum, Enum Variant)
#[derive(Debug, Default)]
pub(crate) struct Attributes {
    pub default_literal: Option<TokenStream>,
    pub values: BTreeMap<String, String>,
}

impl Attributes {
    pub(crate) fn new(attrs: &[syn::Attribute]) -> Self {
        let mut default_literal: Option<TokenStream> = None;
        let mut values = BTreeMap::<String, String>::new();

        attrs
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
                                default_literal = get_default_token_stream(&mut group_iter);
                            }
                            // Bool Attributes
                            READONLY_ATTR | HIDDEN_ATTR | OFFLINE_ONLY_ATTR | TRANSIENT_ATTR
                            | RUNTIME_ONLY_ATTR | IGNORE_DEPS_ATTR => {
                                values.insert(ident, "true".into());
                            }
                            // ResourceType Attribute
                            RESOURCE_TYPE_ATTR => {
                                values.insert(ident, get_resource_type(&mut group_iter));
                            }
                            // Literal Attributes
                            EDITOR_TYPE_ATTR | GROUP_ATTR | TOOLTIP_ATTR => {
                                values.insert(ident, get_attribute_literal(&mut group_iter));
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

        Self {
            default_literal,
            values,
        }
    }

    /// Generate the Attributes descriptor as key-value entries in dictionary.
    pub(crate) fn generate_descriptor_impl(&self) -> TokenStream {
        let attributes: Vec<TokenStream> = self
            .values
            .iter()
            .map(|(k, v)| {
                quote! {  attr.insert(String::from(#k),String::from(#v)); }
            })
            .collect();

        if attributes.is_empty() {
            quote! { None }
        } else {
            quote! {
                {
                    let mut attr = std::collections::HashMap::new();
                    #(#attributes)*
                    Some(attr)
                }
            }
        }
    }
}

// Extract the literal attribute (ex: tooltip = "tooltip info as a string")
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

// Retrieive a type path for the resource_type property (resource_type = crate::module::runtime_type)
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
