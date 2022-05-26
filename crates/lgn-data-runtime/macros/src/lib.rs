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

        impl lgn_data_runtime::ResourceDescriptor for #name {
            const TYPENAME: &'static str = #str_value;
        }

        impl lgn_data_runtime::Resource for #name {
            fn as_reflect(&self) -> &dyn lgn_data_model::TypeReflection {
                self
            }
            fn as_reflect_mut(&mut self) -> &mut dyn lgn_data_model::TypeReflection {
                self
            }

            fn clone_dyn(&self) -> Box<dyn Resource> {
                Box::new(self.clone())
            }
            fn get_meta(&self) -> Option<&lgn_data_runtime::Metadata> {
                Some(&self.meta)
            }
            fn get_meta_mut(&mut self) -> Option<&mut lgn_data_runtime::Metadata> {
                Some(&mut self.meta)
            }
        }

        impl lgn_data_model::TypeReflection for #name {
            fn get_type(&self) -> lgn_data_model::TypeDefinition {
                Self::get_type_def()
            }
            fn get_type_def() -> lgn_data_model::TypeDefinition {
                lgn_data_model::TypeDefinition::None
            }
        }

    };

    resource.extend(proc_macro::TokenStream::from(resource_impl));
    resource
}
