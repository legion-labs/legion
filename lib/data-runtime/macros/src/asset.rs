use proc_macro::TokenStream;
use quote::quote;

pub fn derive_resource(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let _gen = quote! {
        impl Resource for #name {}
    };
    //gen.into()
    TokenStream::new()
}
