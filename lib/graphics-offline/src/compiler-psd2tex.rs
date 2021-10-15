use std::env;

use legion_data_compiler::{
    compiler_api::{
        compiler_main, CompilationOutput, CompilerContext, CompilerDescriptor, CompilerError,
        DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use legion_data_runtime::Resource;
use legion_graphics_offline::PsdFile;

static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &(
        legion_graphics_offline::PsdFile::TYPE,
        legion_graphics_offline::Texture::TYPE,
    ),
    compiler_hash_func: hash_code_and_data,
    compile_func: compile,
};

fn compile(mut context: CompilerContext) -> Result<CompilationOutput, CompilerError> {
    let resources = context
        .take_registry()
        .add_loader::<legion_graphics_offline::PsdFile>()
        .create();
    let mut resources = resources.lock().unwrap();

    let resource =
        resources.load_sync::<legion_graphics_offline::PsdFile>(context.source.content_id());

    let resource = resource.get(&resources).unwrap();

    let mut compiled_resources = vec![];

    let compiled_content = {
        let final_image = resource
            .final_texture()
            .ok_or(CompilerError::CompilationError(
                "Failed to generate texture",
            ))?;
        serde_json::to_vec(&final_image)
            .map_err(|_e| CompilerError::CompilationError("Failed to serialize"))?
    };

    let output = context.store(&compiled_content, context.target_unnamed.clone())?;
    compiled_resources.push(output);

    let compile_layer = |psd: &PsdFile, layer_name| -> Result<Vec<u8>, CompilerError> {
        let image = psd.layer_texture(layer_name).unwrap();
        serde_json::to_vec(&image)
            .map_err(|_e| CompilerError::CompilationError("Failed to serialize"))
    };

    for layer_name in resource
        .layer_list()
        .ok_or(CompilerError::CompilationError(
            "Failed to extract layer names",
        ))?
    {
        let pixels = compile_layer(resource, layer_name)?;
        let output = context.store(&pixels, context.target_unnamed.new_named(layer_name))?;
        compiled_resources.push(output);
    }

    Ok(CompilationOutput {
        compiled_resources,
        resource_references: vec![],
    })
}

fn main() {
    std::process::exit(match compiler_main(env::args(), &COMPILER_INFO) {
        Ok(_) => 0,
        Err(_) => 1,
    });
}
