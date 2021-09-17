use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::*;
use syn::*;

pub fn declare_queue_impl(input: TokenStream) -> TokenStream {
    let ast = parse::<DeriveInput>(input).unwrap();
    let struct_identifier = ast.ident.clone();

    let push_methods = ast.generics.params.iter().map(|p| match p {
        GenericParam::Type(t) => {
            let snake_type = t.ident.to_string().to_case(Case::Snake);
            let push_id = format_ident!("push_{}", snake_type);
            quote! {
                pub fn #push_id( &self, value: #t ){
                }
            }
        }
        GenericParam::Lifetime(_) => panic!("lifetime of generic param not supported"),
        GenericParam::Const(_) => panic!("const generic param not supported"),
    });

    TokenStream::from(quote! {
        struct #struct_identifier {
            buffer: Vec<u8>,
        }

        impl #struct_identifier {
            pub fn new(buffer_size: usize) -> Self {
                let mut buffer: Vec<u8> = Vec::new();
                buffer.reserve(buffer_size);
                Self { buffer }
            }

            #(#push_methods)*
        }
    })
}
