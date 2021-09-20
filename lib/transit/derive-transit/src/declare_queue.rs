use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::*;
use syn::*;

fn gen_push_methods(type_args: &[syn::Ident]) -> Vec<quote::__private::TokenStream> {
    let mut value_tupe_counter: u8 = 0;
    type_args.iter().map(|value_type_id| {
            let index = value_tupe_counter;
            value_tupe_counter += 1;
            let snake_type = value_type_id.to_string().to_case(Case::Snake);
            let push_id = format_ident!("push_{}", snake_type);
            quote! {
                pub fn #push_id( &mut self, value: #value_type_id ){
                    self.buffer.push(#index);
                    let buffer_size_before = self.buffer.len();
                    if <#value_type_id as transit::Serialize>::is_size_static(){
                        <#value_type_id as transit::Serialize>::write_value( &mut self.buffer, &value );
                        assert!( self.buffer.len() == buffer_size_before + std::mem::size_of::<#value_type_id>());
                    }
                    else{
                        // we force the dynamically sized object to first serialize their size as unsigned 32 bits
                        // this will allow unparsable objects to be skipped by the reader
                        let value_size = <#value_type_id as transit::Serialize>::get_value_size( &value ).unwrap();
                        transit::write_pod( &mut self.buffer, &value_size );
                        <#value_type_id as transit::Serialize>::write_value( &mut self.buffer, &value );
                        assert!( self.buffer.len() == buffer_size_before + std::mem::size_of::<u32>() + value_size as usize);
                    }
                }
            }
    }).collect()
}

fn gen_read_method(
    type_args: &[syn::Ident],
    any_ident: &syn::Ident,
) -> quote::__private::TokenStream {
    let mut value_tupe_counter: u8 = 0;
    let type_index_cases = type_args.iter().map(|value_type_id| {
        let index = value_tupe_counter;
        value_tupe_counter += 1;
        quote! {
            #index => {
                unsafe{
                    let begin_obj = self.buffer.as_ptr().add( offset+1 );
                    #any_ident::#value_type_id( read_pod::<#value_type_id>(begin_obj) )
                }
            },
        }
    });

    quote! {
        pub fn read_value_at_offset( &self, offset: usize ) -> #any_ident{
            let index = self.buffer[offset];
            match index{
                #(#type_index_cases)*
                _ => {
                    panic!("any other");
                }
            }
        }
    }
}

pub fn declare_queue_impl(input: TokenStream) -> TokenStream {
    let ast = parse::<DeriveInput>(input).unwrap();
    let struct_identifier = ast.ident.clone();

    let type_args: Vec<syn::Ident> = ast
        .generics
        .params
        .iter()
        .map(|p| match p {
            GenericParam::Type(t) => t.ident.clone(),
            GenericParam::Lifetime(_) => panic!("lifetime of generic param not supported"),
            GenericParam::Const(_) => panic!("const generic param not supported"),
        })
        .collect();

    let push_methods = gen_push_methods(&type_args);
    let any_ident = format_ident!("{}Any", struct_identifier);
    let read_method = gen_read_method(&type_args, &any_ident);

    TokenStream::from(quote! {

        enum #any_ident{
            #(#type_args(#type_args),)*
        }

        struct #struct_identifier {
            buffer: Vec<u8>,
        }

        impl #struct_identifier {
            pub fn new(buffer_size: usize) -> Self {
                let mut buffer: Vec<u8> = Vec::new();
                buffer.reserve(buffer_size);
                Self { buffer }
            }

            pub fn len_bytes(&self) -> usize{
                self.buffer.len()
            }

            #read_method

            #(#push_methods)*

        }
    })
}
