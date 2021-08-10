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
use legion_resources::{test_resource, ResourceId, ResourceRegistry};

static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    resource_types: &[test_resource::TYPE_ID],
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
    dependencies: &[ResourceId],
    _target: Target,
    _platform: Platform,
    _locale: &Locale,
    compiled_asset_store_path: CompiledAssetStoreAddr,
    resource_dir: &Path,
) -> Result<CompilationOutput, CompilerError> {
    let mut resources = ResourceRegistry::default();
    resources.register_type(
        test_resource::TYPE_ID,
        Box::new(test_resource::TestResourceProc {}),
    );

    let mut asset_store = LocalCompiledAssetStore::open(compiled_asset_store_path)
        .ok_or(CompilerError::AssetStoreError)?;

    let guid = primary_asset_id(source, test_asset::TYPE_ID);

    let resource = compiler_load_resource(source, resource_dir, &mut resources)?;
    let resource = resource
        .get::<test_resource::TestResource>(&resources)
        .unwrap();

    let compiled_asset = {
        let mut content = resource.content.as_bytes().to_owned();
        content.reverse();
        content
    };

    let checksum = asset_store
        .store(&compiled_asset)
        .ok_or(CompilerError::AssetStoreError)?;

    let asset = CompiledAsset {
        guid,
        checksum,
        size: compiled_asset.len(),
    };

    // in this test example every build dependency becomes a asset_reference/load-time dependency.
    let asset_references: Vec<_> = dependencies
        .iter()
        .map(|dep| (guid, primary_asset_id(*dep, test_asset::TYPE_ID)))
        .collect();

    Ok(CompilationOutput {
        compiled_assets: vec![asset],
        asset_references,
    })
}

fn main() {
    std::process::exit(match compiler_main(env::args(), &COMPILER_INFO) {
        Ok(_) => 0,
        Err(_) => 1,
    });
}
