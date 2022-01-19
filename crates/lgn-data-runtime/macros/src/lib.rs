//! Procedural macros associated with runtime asset management module of data
//! processing pipeline.

// crate-specific lint exceptions:
#![warn(missing_docs)]

use std::stringify;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct, LitStr};

/// Derives a default implementation of the Resource trait for a type.
#[proc_macro_attribute]
pub fn resource(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut resource = item.clone();
    let item = parse_macro_input!(item as ItemStruct);

    let str_value = parse_macro_input!(attr as LitStr);
    let name = item.ident;
    let resource_impl = quote! {
        impl Resource for #name {
            const TYPENAME: &'static str = #str_value;
        }
    };

    resource.extend(proc_macro::TokenStream::from(resource_impl));
    resource
}
