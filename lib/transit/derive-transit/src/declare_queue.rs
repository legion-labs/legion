use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::*;
use syn::*;

pub fn declare_queue_impl(input: TokenStream) -> TokenStream {
    let ast = parse::<DeriveInput>(input).unwrap();
    let struct_identifier = ast.ident.clone();

    let mut value_tupe_counter: u8 = 0;
    let push_methods = ast.generics.params.iter().map(|p| match p {
        GenericParam::Type(t) => {
            let index = value_tupe_counter;
            value_tupe_counter += 1;
            let value_type_id = &t.ident;
            let snake_type = value_type_id.to_string().to_case(Case::Snake);
            let push_id = format_ident!("push_{}", snake_type);
            quote! {
                pub fn #push_id( &mut self, value: #t ){
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

            pub fn len_bytes(&self) -> usize{
                self.buffer.len()
            }

            #(#push_methods)*
        }
    })
}
