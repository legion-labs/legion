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
    CompiledResource, CompilerHash, Locale, Platform, Target,
};
use legion_resources::ResourceRegistry;

static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &(text_resource::TEXT_RESOURCE, text_resource::TEXT_RESOURCE),
    compiler_hash_func: compiler_hash,
    compile_func: compile,
};

fn compiler_hash(
    code: &'static str,
    data: &'static str,
    _target: Target,
    _platform: Platform,
    _locale: Locale,
) -> Vec<CompilerHash> {
    let mut hasher = DefaultHasher::new();
    code.hash(&mut hasher);
    data.hash(&mut hasher);
    vec![CompilerHash(hasher.finish())]
}

fn compile(context: CompilerContext) -> Result<CompilationOutput, CompilerError> {
    let mut resources = ResourceRegistry::default();
    resources.register_type(
        text_resource::TEXT_RESOURCE,
        Box::new(text_resource::TextResourceProc {}),
    );

    let handle = context.load_resource(
        &context.compile_path.direct_dependency().unwrap(),
        &mut resources,
    )?;
    let mut resource = handle
        .get_mut::<text_resource::TextResource>(&mut resources)
        .unwrap();

    resource.content = resource.content.chars().rev().collect();

    let mut bytes = vec![];

    let (nbytes, _) = resources
        .serialize_resource(text_resource::TEXT_RESOURCE, &handle, &mut bytes)
        .map_err(CompilerError::ResourceWriteFailed)?;

    let checksum = context
        .content_store
        .store(&bytes)
        .ok_or(CompilerError::AssetStoreError)?;

    let asset = CompiledResource {
        path: context.compile_path,
        checksum,
        size: nbytes,
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
