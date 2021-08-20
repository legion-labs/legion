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
    transform: &(refs_resource::TYPE_ID, refs_resource::TYPE_ID),
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
        refs_resource::TYPE_ID,
        Box::new(refs_resource::TestResourceProc {}),
    );

    let resource = context.load_resource(
        &context.derived.direct_dependency().unwrap(),
        &mut resources,
    )?;
    let resource = resource
        .get::<refs_resource::TestResource>(&resources)
        .unwrap();

    let compiled_asset = {
        let mut content = resource.content.as_bytes().to_owned();
        content.reverse();
        content
    };

    let checksum = context
        .content_store
        .store(&compiled_asset)
        .ok_or(CompilerError::AssetStoreError)?;

    let asset = CompiledResource {
        path: context.derived.clone(),
        checksum,
        size: compiled_asset.len(),
    };

    // in this test example every build dependency becomes a reference/load-time dependency.
    let derived = context.derived.clone();
    let asset_references: Vec<_> = context
        .dependencies
        .iter()
        .map(|dep| (derived.clone(), dep.clone()))
        .collect();

    Ok(CompilationOutput {
        compiled_resources: vec![asset],
        resource_references: asset_references,
    })
}

fn main() {
    std::process::exit(match compiler_main(env::args(), &COMPILER_INFO) {
        Ok(_) => 0,
        Err(_) => 1,
    });
}
