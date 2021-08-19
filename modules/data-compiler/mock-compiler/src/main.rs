use std::{
    collections::hash_map::DefaultHasher,
    env,
    hash::{Hash, Hasher},
    path::Path,
};

use legion_content_store::{ContentStore, ContentStoreAddr, HddContentStore};

use legion_data_compiler::{
    compiler_api::{
        compiler_load_resource, compiler_main, CompilationOutput, CompilerDescriptor,
        CompilerError, DATA_BUILD_VERSION,
    },
    CompiledResource, CompilerHash, Locale, Platform, Target,
};
use legion_resources::{ResourcePathId, ResourceRegistry};

static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transforms: &[(mock_resource::TYPE_ID, mock_resource::TYPE_ID)],
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

fn compile(
    derived: ResourcePathId,
    _dependencies: &[ResourcePathId],
    _target: Target,
    _platform: Platform,
    _locale: &Locale,
    compiled_asset_store_path: ContentStoreAddr,
    resource_dir: &Path,
) -> Result<CompilationOutput, CompilerError> {
    let mut resources = ResourceRegistry::default();
    resources.register_type(
        mock_resource::TYPE_ID,
        Box::new(mock_resource::MockResourceProc {}),
    );

    let mut content_store =
        HddContentStore::open(compiled_asset_store_path).ok_or(CompilerError::AssetStoreError)?;

    // todo: source_resource is wrong
    let resource = compiler_load_resource(derived.source_resource(), resource_dir, &mut resources)?;
    let resource = resource
        .get::<mock_resource::MockResource>(&resources)
        .unwrap();

    let magic_value = resource.magic_value * 2;
    let compiled_asset = magic_value.to_ne_bytes();

    let checksum = content_store
        .store(&compiled_asset)
        .ok_or(CompilerError::AssetStoreError)?;

    let asset = CompiledResource {
        path: derived,
        checksum,
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
