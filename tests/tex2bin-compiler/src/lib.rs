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
use lgn_graphics_data::runtime_texture;

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

fn compile(mut context: CompilerContext<'_>) -> Result<CompilationOutput, CompilerError> {
    let resources = context.registry();

    let resource = resources
        .load_sync::<lgn_graphics_data::offline_texture::Texture>(context.source.resource_id());
    let resource = resource.get(&resources).unwrap();

    let mut compiled_asset = vec![];
    runtime_texture::Texture::compile_from_offline(
        resource.width,
        resource.height,
        resource.format,
        resource.quality,
        resource.color_channels,
        &resource.rgba,
        &mut compiled_asset,
    );

    let asset = context.store(&compiled_asset, context.target_unnamed.clone())?;

    Ok(CompilationOutput {
        compiled_resources: vec![asset],
        resource_references: vec![],
    })
}
