use std::{
    collections::hash_map::DefaultHasher,
    env,
    hash::{Hash, Hasher},
};

use base64::encode;

use legion_data_compiler::{
    compiler_api::{
        compiler_main, CompilationOutput, CompilerContext, CompilerDescriptor, CompilerError,
        DATA_BUILD_VERSION,
    },
    CompiledResource, CompilerHash, Locale, Platform, Target,
};
use legion_data_offline::resource::ResourceRegistryOptions;

static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &(binary_resource::TYPE_ID, text_resource::TYPE_ID),
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
        .add_type(
            binary_resource::TYPE_ID,
            Box::new(binary_resource::BinaryResourceProc {}),
        )
        .create_registry();

    let resource = context.load_resource(
        &context.compile_path.direct_dependency().unwrap(),
        &mut resources,
    )?;
    let resource = resource
        .get::<binary_resource::BinaryResource>(&resources)
        .unwrap();

    let base64string = encode(&resource.content);
    let compiled_asset = base64string.as_bytes();

    let checksum = context
        .content_store
        .store(compiled_asset)
        .ok_or(CompilerError::AssetStoreError)?;

    let asset = CompiledResource {
        path: context.compile_path,
        checksum: checksum.into(),
        size: compiled_asset.len(),
    };

    // in this mock build dependency are _not_ runtime references.
    Ok(CompilationOutput {
        compiled_resources: vec![asset],
        resource_references: vec![],
    })
}

fn main() {
    std::process::exit(match compiler_main(env::args(), &COMPILER_INFO) {
        Ok(_) => 0,
        Err(_) => 1,
    });
}
