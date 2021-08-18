use legion_macro_utils::LegionManifest;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Path};

pub fn derive_render_resource(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let manifest = LegionManifest::default();

    let legion_render_path: Path = manifest.get_path(crate::modules::LEGION_RENDER);
    let legion_asset_path: Path = manifest.get_path(crate::modules::LEGION_ASSET);
    let legion_core_path: Path = manifest.get_path(crate::modules::LEGION_CORE);
    let struct_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics #legion_render_path::renderer::RenderResource for #struct_name #type_generics #where_clause {
            fn resource_type(&self) -> Option<#legion_render_path::renderer::RenderResourceType> {
                Some(#legion_render_path::renderer::RenderResourceType::Buffer)
            }
            fn write_buffer_bytes(&self, buffer: &mut [u8]) {
                use #legion_core_path::Bytes;
                self.write_bytes(buffer);
            }
            fn buffer_byte_len(&self) -> Option<usize> {
                use #legion_core_path::Bytes;
                Some(self.byte_len())
            }
            fn texture(&self) -> Option<&#legion_asset_path::Handle<#legion_render_path::texture::Texture>> {
                None
            }

        }
    })
}
