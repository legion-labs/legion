use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::reflection::DataContainerMetaInfo;

/// Generate code `AssetLoader` Runtime Registration
pub fn generate_registration_code(structs: &[DataContainerMetaInfo]) -> TokenStream {
    let entries: Vec<TokenStream> = structs
        .iter()
        .map(|struct_meta| {
            let type_name = format_ident!("{}", &struct_meta.name);
            quote! { .add_loader::<#type_name>() }
        })
        .collect();
    quote! {
        pub fn add_loaders(registry: lgn_data_runtime::AssetRegistryOptions) -> lgn_data_runtime::AssetRegistryOptions {
            registry
            #(#entries)*
        }
    }
}

pub fn generate(data_container_info: &DataContainerMetaInfo, add_uses: bool) -> TokenStream {
    let runtime_identifier = format_ident!("{}", data_container_info.name);
    let runtime_name = format!("runtime_{}", data_container_info.name).to_lowercase();
    let runtime_loader = format_ident!("{}Loader", data_container_info.name);

    let life_time = if data_container_info.need_life_time() {
        quote! {<'r>}
    } else {
        quote! {}
    };

    let use_quotes = if add_uses {
        quote! {
        use std::{any::Any, io};
        use lgn_data_runtime::{Asset, AssetLoader,Resource};
        }
    } else {
        quote! {}
    };

    quote! {

        #use_quotes

        impl #life_time Resource for #runtime_identifier #life_time {
            const TYPENAME: &'static str = #runtime_name;
        }

        impl #life_time Asset for #runtime_identifier #life_time {
            type Loader = #runtime_loader;
        }

        #[derive(Default)]
        pub struct #runtime_loader {}

        impl AssetLoader for #runtime_loader {
            fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
                let output : #runtime_identifier = bincode::deserialize_from(reader).map_err(|_err|
                    io::Error::new(io::ErrorKind::InvalidData, "Failed to parse"))?;
                Ok(Box::new(output))
            }

            fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
        }
    }
}
