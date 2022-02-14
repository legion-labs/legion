// crate-specific lint exceptions:
//#![allow()]

use lgn_data_compiler::{
    compiler_api::{
        CompilationOutput, CompilerContext, CompilerDescriptor, CompilerError, DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use lgn_data_offline::{resource::ResourceProcessor, Transform};
use lgn_data_runtime::{AssetRegistryOptions, Resource};
use lgn_graphics_data::offline_texture::TextureProcessor;

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        lgn_graphics_data::offline_png::PngFile::TYPE,
        lgn_graphics_data::offline_texture::Texture::TYPE,
    ),
    init_func: init,
    compiler_hash_func: hash_code_and_data,
    compile_func: compile,
};

fn init(options: AssetRegistryOptions) -> AssetRegistryOptions {
    options.add_loader::<lgn_graphics_data::offline_png::PngFile>()
}

fn compile(mut context: CompilerContext<'_>) -> Result<CompilationOutput, CompilerError> {
    let asset_registry = context.registry();

    let resource_handle = asset_registry
        .load_sync::<lgn_graphics_data::offline_png::PngFile>(context.source.resource_id());

    if let Some(err) = asset_registry.retrieve_err(resource_handle.id()) {
        return Err(CompilerError::CompilationError(err.to_string()));
    }

    let texture = resource_handle
        .get(&asset_registry)
        .ok_or_else(|| {
            CompilerError::CompilationError(format!(
                "Failed to retrieve resource '{}'",
                context.source.resource_id()
            ))
        })?
        .as_texture();

    let mut content = vec![];
    let texture_proc = TextureProcessor {};
    texture_proc
        .write_resource(&texture, &mut content)
        .unwrap_or_else(|_| panic!("writing to file {}", context.source.resource_id()));

    let output = context.store(&content, context.target_unnamed.clone())?;

    Ok(CompilationOutput {
        compiled_resources: vec![output],
        resource_references: vec![],
    })
}
