use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use legion_data_compiler::{
    compiler_api::{CompilationOutput, CompilerContext, CompilerError},
    CompiledResource, CompilerHash, Locale, Platform, Target,
};
use legion_data_offline::resource::{Resource, ResourceRegistryOptions};
use legion_data_runtime::Asset;
use sample_data_compiler::{offline_data::CompilableResource, runtime_data::CompilableAsset};
use serde::Serialize;

use crate::offline_to_runtime::FromOffline;

pub fn compiler_hash(
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

pub fn compile<OfflineType, RuntimeType>(
    context: CompilerContext<'_>,
) -> Result<CompilationOutput, CompilerError>
where
    OfflineType: Resource + CompilableResource,
    RuntimeType: Asset + CompilableAsset + FromOffline<OfflineType> + Serialize,
{
    let mut resources = ResourceRegistryOptions::new()
        .add_type(
            OfflineType::TYPE_ID,
            Box::new(OfflineType::Processor::default()),
        )
        .create_registry();

    let resource = context.load_resource(
        &context.compile_path.direct_dependency().unwrap(),
        &mut resources,
    )?;
    let resource = resource.get::<OfflineType>(&resources).unwrap();

    let asset = RuntimeType::from_offline(resource);
    let compiled_asset = bincode::serialize(&asset).unwrap();

    let checksum = context
        .content_store
        .store(&compiled_asset)
        .ok_or(CompilerError::AssetStoreError)?;

    let asset = CompiledResource {
        path: context.compile_path,
        checksum: checksum.into(),
        size: compiled_asset.len(),
    };

    Ok(CompilationOutput {
        compiled_resources: vec![asset],
        resource_references: vec![],
    })
}
