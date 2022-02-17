//! Legion Derive
//!
//! TODO: write documentation.

// crate-specific lint exceptions:
//#![allow()]

mod app_plugin;
mod legion_main;

use lgn_macro_utils::{derive_label, LegionManifest};
use proc_macro::TokenStream;
use quote::format_ident;

/// Generates a dynamic plugin entry point function for the given `Plugin` type.  
#[proc_macro_derive(DynamicPlugin)]
pub fn derive_dynamic_plugin(input: TokenStream) -> TokenStream {
    app_plugin::derive_dynamic_plugin(input)
}

#[proc_macro_attribute]
pub fn legion_main(attr: TokenStream, item: TokenStream) -> TokenStream {
    legion_main::legion_main(attr, item)
}

#[proc_macro_derive(AppLabel)]
pub fn derive_app_label(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let mut trait_path = LegionManifest::default().get_path("lgn_app");
    trait_path.segments.push(format_ident!("AppLabel").into());
    derive_label(input, &trait_path)
}
