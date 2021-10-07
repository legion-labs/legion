use std::{
    collections::hash_map::DefaultHasher,
    env,
    hash::{Hash, Hasher},
};

use legion_data_compiler::{
    compiler_api::{
        compiler_main, CompilationOutput, CompilerContext, CompilerDescriptor, CompilerError,
        DATA_BUILD_VERSION,
    },
    CompilerHash, Locale, Platform, Target,
};
use legion_data_offline::resource::ResourceRegistryOptions;
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
    compiler_hash_func: compiler_hash,
    compile_func: compile,
};

fn compiler_hash(
    code: &'static str,
    data: &'static str,
    _target: Target,
    _platform: Platform,
    _locale: &Locale,
) -> CompilerHash {
    let mut hasher = DefaultHasher::new();
    code.hash(&mut hasher);
    data.hash(&mut hasher);
    CompilerHash(hasher.finish())
}

fn compile(context: CompilerContext) -> Result<CompilationOutput, CompilerError> {
    let mut resources = ResourceRegistryOptions::new()
        .add_type::<legion_graphics_offline::PsdFile>()
        .create_registry();

    let resource = context.load_resource(
        &context.compile_path.direct_dependency().unwrap(),
        &mut resources,
    )?;
    let resource = resource
        .get::<legion_graphics_offline::PsdFile>(&resources)
        .unwrap();

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

    compiled_resources.push(CompilerContext::store(
        context.content_store,
        &compiled_content,
        context.compile_path.clone(),
    )?);

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
        compiled_resources.push(CompilerContext::store(
            context.content_store,
            &pixels,
            context.compile_path.new_named(layer_name),
        )?);
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
