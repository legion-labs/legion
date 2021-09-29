// ['DataContainer'] serialization

use proc_macro::TokenStream;
use quote::*;
use syn::{parse_macro_input, DeriveInput};
type QuoteRes = quote::__private::TokenStream;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use syn::Meta::{List, NameValue, Path};
use syn::NestedMeta::{Lit, Meta};

#[derive(Copy, Clone)]
pub struct Symbol(&'static str);

pub const LEGION_TAG: Symbol = Symbol("legion");
pub const DEFAULT_ATTR: Symbol = Symbol("default");
pub const HIDDEN_ATTR: Symbol = Symbol("hidden");
pub const OFFLINE_ATTR: Symbol = Symbol("offline");
pub const TOOLTIP_ATTR: Symbol = Symbol("tooltip");
pub const READONLY_ATTR: Symbol = Symbol("readonly");
pub const CATEGORY_ATTR: Symbol = Symbol("category");

impl PartialEq<Symbol> for syn::Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl<'a> PartialEq<Symbol> for &'a syn::Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl PartialEq<Symbol> for syn::Ident {
    fn eq(&self, word: &Symbol) -> bool {
        self == word.0
    }
}

impl<'a> PartialEq<Symbol> for &'a syn::Ident {
    fn eq(&self, word: &Symbol) -> bool {
        *self == word.0
    }
}

fn get_lit_str(lit: &syn::Lit) -> Result<&syn::LitStr, ()> {
    if let syn::Lit::Str(lit) = lit {
        Ok(lit)
    } else {
        Err(())
    }
}

fn metadata_from_type(t: &syn::Type) -> Option<QuoteRes> {
    match t {
        syn::Type::BareFn(fun) => Some(quote! {#fun}),
        syn::Type::Path(type_path) => Some(quote! {#type_path}),
        syn::Type::Reference(reference) => Some(quote! {#reference}),
        _ => None,
    }
}

// Extract all `#[legion(...)]` attributes
fn get_legion_meta_items(attr: &syn::Attribute) -> Result<Vec<syn::NestedMeta>, ()> {
    if attr.path != LEGION_TAG {
        return Ok(Vec::new());
    }

    match attr.parse_meta() {
        Ok(List(meta)) => Ok(meta.nested.into_iter().collect()),
        Ok(_other) => {
            panic!("Legion proc-macro: expected attributes syntax #[legion(...)");
        }
        Err(err) => {
            panic!("Legion proc-macro: Error parsing attributes, {}", err);
        }
    }
}

#[derive(Default)]
struct MemberMetaInfo {
    name: String,
    type_name: QuoteRes,
    offline: bool,
    category: Option<String>,
    hidden: bool,
    readonly: bool,
    tooltip: Option<String>,
    default_literal: Option<QuoteRes>,
}

impl MemberMetaInfo {
    fn calculate_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.name.hash(&mut hasher);
        format!("{}", self.type_name).hash(&mut hasher);
        //self.type_name.hash(&mut hasher);
        hasher.finish()
    }
}

fn get_member_info(field: &syn::Field) -> Option<MemberMetaInfo> {
    let field_type = metadata_from_type(&field.ty)?;

    let mut member_info = MemberMetaInfo {
        name: field.ident.as_ref().unwrap().to_string(),
        type_name: field_type,
        category: None,
        offline: false,
        hidden: false,
        readonly: false,
        tooltip: None,
        default_literal: None,
    };

    for meta_item in field
        .attrs
        .iter()
        .flat_map(|attr| get_legion_meta_items(attr))
        .flatten()
    {
        match &meta_item {
            // Parse `#[legion(offline)]`
            Meta(Path(word)) if word == OFFLINE_ATTR => {
                member_info.offline = true;
            }

            // Parse `#[legion(hidden)]`
            Meta(Path(word)) if word == HIDDEN_ATTR => {
                member_info.hidden = true;
            }

            // Parse `#[legion(readonly)]`
            Meta(Path(word)) if word == READONLY_ATTR => {
                member_info.readonly = true;
            }

            // Parse `#[legion(category = "categoryName")]`
            Meta(NameValue(m)) if m.path == CATEGORY_ATTR => {
                if let Ok(category) = get_lit_str(&m.lit) {
                    member_info.category = Some(category.value());
                }
            }

            // Parse `#[legion(tooltip = "Tool Tip String")]`
            Meta(NameValue(m)) if m.path == TOOLTIP_ATTR => {
                if let Ok(tooltip) = get_lit_str(&m.lit) {
                    member_info.tooltip = Some(tooltip.value());
                }
            }

            // Parse `#[legion(default = "...")]`
            Meta(NameValue(m)) if m.path == DEFAULT_ATTR => {
                member_info.default_literal = Some(m.lit.clone().into_token_stream());
            }

            Meta(meta_item) => {
                let path = meta_item
                    .path()
                    .into_token_stream()
                    .to_string()
                    .replace(' ', "");
                panic!(
                    "Legion proc-macro: unknown legion container attribute `{}`",
                    path
                );
            }

            Lit(_lit) => {
                panic!("Legion proc-macro: unexpected literal in legion container attribute");
            }
        }
    }
    Some(member_info)
}

pub fn derive_data_container(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let offline_identifer = ast.ident.clone();
    let offline_name = format!("{}", ast.ident);
    let runtime_ident = format_ident!("Runtime{}", offline_identifer);
    let mut need_life_time = false;
    let mut members = Vec::new();

    // Hasher for the class signature
    // Class Name, Member Name, Member Type, Default Value Attribute
    let mut hasher = DefaultHasher::new();
    offline_identifer.hash(&mut hasher);

    match ast.data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(named_fields) => {
                for field in named_fields.named {
                    if let Some(member_info) = get_member_info(&field) {
                        hasher.write_u64(member_info.calculate_hash());
                        if format!("{}", member_info.type_name) == "String" {
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

    let runtime_members = members.iter().filter(|m| !m.offline).map(|m| {
        let member_ident = format_ident!("{}", &m.name);
        let member_type = &m.type_name;
        if format!("{}", m.type_name) == "String" {
            quote! { #member_ident : &'r str, }
        } else {
            quote! { #member_ident : #member_type, }
        }
    });

    let runtime_members_defaults = members.iter().filter(|m| !m.offline).map(|m| {
        let member_ident = format_ident!("{}", &m.name);
        if let Some(default_value) = &m.default_literal {
            quote! { #member_ident : #default_value, }
        } else {
            quote! { #member_ident : Default::default(), }
        }
    });

    let offline_members_defaults = members.iter().map(|m| {
        let member_ident = format_ident!("{}", &m.name);
        if let Some(default_value) = &m.default_literal {
            // If the default is a string literal, add "into()" to convert to String
            match syn::parse::<syn::Lit>(default_value.clone().into()) {
                Ok(syn::Lit::Str(_) | syn::Lit::ByteStr(_)) => {
                    quote! { #member_ident : #default_value.into(),}
                }
                _ => quote! { #member_ident : #default_value, },
            }
        } else {
            quote! { #member_ident : Default::default(), }
        }
    });

    let offline_members_json_reads = members.iter().map(|m| {
        let member_ident = format_ident!("{}", &m.name);
        let member_name = &m.name;
        quote! {
            if let Some(value) = values.get(#member_name) {
                new_obj.#member_ident = serde_json::from_value(value.clone()).unwrap();
            }
        }
    });

    let offline_members_json_writes = members.iter().map(|m| {
        let member_ident = format_ident!("{}", &m.name);
        let prop_id = format!("\t\"{}\" : ", &m.name);

        if let Some(default_value) = &m.default_literal {
            quote! {
                if self.#member_ident != #default_value {
                    if let Ok(json_string) = serde_json::to_string(&self.#member_ident) {
                        writer.write_all(",\n".as_bytes());
                        writer.write_all(#prop_id.as_bytes());
                        writer.write_all(&json_string.as_bytes());
                    }
                }
            }
        } else {
            quote! {
                if let Ok(json_string) = serde_json::to_string(&self.#member_ident) {
                    writer.write_all(",\n".as_bytes());
                    writer.write_all(#prop_id.as_bytes());
                    writer.write_all(&json_string.as_bytes());
                }
            }
        }
    });

    // Optional lifetime parameters
    let life_time = match need_life_time {
        true => quote! {<'r>},
        _ => quote! {},
    };

    let signature_hash = hasher.finish();

    TokenStream::from(quote! {

        // Runtime Structure
        #[derive(Debug, Serialize, Deserialize)]
        #[repr(C)]
        struct #runtime_ident#life_time {
            #(#runtime_members)*
        }

        // Runtime default implementation
        impl#life_time Default for #runtime_ident#life_time {
            fn default() -> Self {
                Self {
                    #(#runtime_members_defaults)*
                }
            }
        }
        // Offline default implementation
        impl Default for #offline_identifer {
            fn default() -> Self {
                Self {
                    #(#offline_members_defaults)*
                }
            }
        }

        // Offline Json serialization
        impl #offline_identifer {

            fn create_from_json(json_data: &str) -> #offline_identifer {
                let mut new_obj = #offline_identifer { ..Default::default() };
                let values: serde_json::Value = serde_json::from_str(json_data).unwrap();
                if values["_class"] == #offline_name {
                    #(#offline_members_json_reads)*
                }
                new_obj
            }

            fn write_to_json(&self, writer: &mut dyn std::io::Write)  {
                writer.write_all("{\n\t\"_class\" : \"".as_bytes());
                writer.write_all(#offline_name.as_bytes());
                writer.write_all("\"".as_bytes());
                #(#offline_members_json_writes)*
                writer.write_all("\n}\n".as_bytes());
            }

            const fn signature_hash() -> u64 {
                #signature_hash
            }
        }
    })
}
