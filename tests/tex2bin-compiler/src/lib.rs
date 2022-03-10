// crate-specific lint exceptions:
//#![allow()]

use std::env;

use lgn_data_compiler::{
    compiler_api::{
        CompilationOutput, CompilerContext, CompilerDescriptor, CompilerError, DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use lgn_data_offline::Transform;
use lgn_data_runtime::{AssetRegistryOptions, Resource};
use lgn_graphics_data::{runtime_texture, TextureFormat};

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        lgn_graphics_data::offline_texture::Texture::TYPE,
        lgn_graphics_data::runtime_texture::Texture::TYPE,
    ),
    init_func: init,
    compiler_hash_func: hash_code_and_data,
    compile_func: compile,
};

fn init(registry: AssetRegistryOptions) -> AssetRegistryOptions {
    registry.add_loader::<lgn_graphics_data::offline_texture::Texture>()
}

#[lgn_tracing::span_fn]
fn compile(mut context: CompilerContext<'_>) -> Result<CompilationOutput, CompilerError> {
    let resources = context.registry();

    let resource = resources
        .load_sync::<lgn_graphics_data::offline_texture::Texture>(context.source.resource_id());
    let image = resource.get(&resources).unwrap();

    let mut compiled_resources = vec![];

    let pixel_size = image.rgba.len() as u32 / image.width / image.height;
    if pixel_size == 1 {
        let mut compiled_asset = vec![];
        runtime_texture::Texture::compile_from_offline(
            image.width,
            image.height,
            TextureFormat::BC4,
            false,
            false,
            &image.rgba,
            &mut compiled_asset,
        );

        compiled_resources.push(context.store(
            &compiled_asset,
            context.target_unnamed.new_named("Roughness"),
        )?);
        compiled_resources.push(context.store(
            &compiled_asset,
            context.target_unnamed.new_named("Metalness"),
        )?);
    } else {
        let mut compiled_asset_srgb = vec![];
        runtime_texture::Texture::compile_from_offline(
            image.width,
            image.height,
            TextureFormat::BC7,
            true,
            false,
            &image.rgba,
            &mut compiled_asset_srgb,
        );

        compiled_resources.push(context.store(
            &compiled_asset_srgb,
            context.target_unnamed.new_named("Albedo"),
        )?);

        // todo: normal compression doest require bc7 (verify and modify)
        let mut compiled_asset_linear = vec![];
        runtime_texture::Texture::compile_from_offline(
            image.width,
            image.height,
            TextureFormat::BC7,
            false,
            false,
            &image.rgba,
            &mut compiled_asset_linear,
        );
        compiled_resources.push(context.store(
            &compiled_asset_linear,
            context.target_unnamed.new_named("Normal"),
        )?);

        let mut compiled_asset_blended = vec![];
        runtime_texture::Texture::compile_from_offline(
            image.width,
            image.height,
            TextureFormat::BC7,
            true,
            true,
            &image.rgba,
            &mut compiled_asset_blended,
        );

        compiled_resources.push(context.store(
            &compiled_asset_blended,
            context.target_unnamed.new_named("AlbedoBlend"),
        )?);
    }

    Ok(CompilationOutput {
        compiled_resources,
        resource_references: vec![],
    })
}
