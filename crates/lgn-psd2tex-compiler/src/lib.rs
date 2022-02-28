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
use lgn_graphics_data::{offline_psd::PsdFile, offline_texture::TextureProcessor};

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        lgn_graphics_data::offline_psd::PsdFile::TYPE,
        lgn_graphics_data::offline_texture::Texture::TYPE,
    ),
    init_func: init,
    compiler_hash_func: hash_code_and_data,
    compile_func: compile,
};

fn init(options: AssetRegistryOptions) -> AssetRegistryOptions {
    options.add_loader::<lgn_graphics_data::offline_psd::PsdFile>()
}

#[lgn_tracing::span_fn]
fn compile(mut context: CompilerContext<'_>) -> Result<CompilationOutput, CompilerError> {
    let resources = context.registry();

    let resource = resources
        .load_sync::<lgn_graphics_data::offline_psd::PsdFile>(context.source.resource_id());

    let resource = resource.get(&resources).unwrap();

    let mut compiled_resources = vec![];
    let texture_proc = TextureProcessor {};

    let compiled_content = {
        let final_image = resource
            .final_texture()
            .ok_or_else(|| CompilerError::CompilationError("Failed to generate texture".into()))?;
        let mut content = vec![];
        texture_proc
            .write_resource(&final_image, &mut content)
            .unwrap_or_else(|_| panic!("writing to file {}", context.source.resource_id()));
        content
    };

    let output = context.store(&compiled_content, context.target_unnamed.clone())?;
    compiled_resources.push(output);

    let compile_layer = |psd: &PsdFile, layer_name| -> Vec<u8> {
        let image = psd.layer_texture(layer_name).unwrap();
        let mut layer_content = vec![];
        texture_proc
            .write_resource(&image, &mut layer_content)
            .unwrap_or_else(|_| panic!("writing to file, from layer {}", layer_name));
        layer_content
    };

    for layer_name in resource
        .layer_list()
        .ok_or_else(|| CompilerError::CompilationError("Failed to extract layer names".into()))?
    {
        let pixels = compile_layer(&resource, layer_name);
        let output = context.store(&pixels, context.target_unnamed.new_named(layer_name))?;
        compiled_resources.push(output);
    }

    Ok(CompilationOutput {
        compiled_resources,
        resource_references: vec![],
    })
}
