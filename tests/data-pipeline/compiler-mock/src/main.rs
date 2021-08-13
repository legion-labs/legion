use std::{
    collections::hash_map::DefaultHasher,
    env,
    hash::{Hash, Hasher},
    path::Path,
};

use legion_asset_store::compiled_asset_store::{
    CompiledAssetStore, CompiledAssetStoreAddr, LocalCompiledAssetStore,
};
use legion_assets::test_asset;
use legion_data_compiler::{
    compiler_api::{
        compiler_load_resource, compiler_main, primary_asset_id, CompilationOutput,
        CompilerDescriptor, CompilerError, DATA_BUILD_VERSION,
    },
    CompiledAsset, CompilerHash, Locale, Platform, Target,
};
use legion_resources::{ResourceId, ResourceRegistry};

static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transforms: &[(mock_offline::TYPE_ID, mock_offline::TYPE_ID)],
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
    source: ResourceId,
    _dependencies: &[ResourceId],
    _target: Target,
    _platform: Platform,
    _locale: &Locale,
    compiled_asset_store_path: CompiledAssetStoreAddr,
    resource_dir: &Path,
) -> Result<CompilationOutput, CompilerError> {
    let mut resources = ResourceRegistry::default();
    resources.register_type(
        mock_offline::TYPE_ID,
        Box::new(mock_offline::MockResourceProc {}),
    );

    let mut asset_store = LocalCompiledAssetStore::open(compiled_asset_store_path)
        .ok_or(CompilerError::AssetStoreError)?;

    let guid = primary_asset_id(source, test_asset::TYPE_ID);

    let resource = compiler_load_resource(source, resource_dir, &mut resources)?;
    let resource = resource
        .get::<mock_offline::MockResource>(&resources)
        .unwrap();

    let magic_value = resource.magic_value * 2;
    let compiled_asset = magic_value.to_ne_bytes();

    let checksum = asset_store
        .store(&compiled_asset)
        .ok_or(CompilerError::AssetStoreError)?;

    let asset = CompiledAsset {
        guid,
        checksum,
        size: compiled_asset.len(),
    };

    // in this mock build dependency are _not_ runtime references.

    Ok(CompilationOutput {
        compiled_assets: vec![asset],
        asset_references: vec![],
    })
}

fn main() {
    std::process::exit(match compiler_main(env::args(), &COMPILER_INFO) {
        Ok(_) => 0,
        Err(_) => 1,
    });
}
