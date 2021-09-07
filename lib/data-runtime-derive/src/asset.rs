use proc_macro::TokenStream;
use quote::quote;

pub fn derive_asset(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl Asset for #name {}
    };
    gen.into()
}
