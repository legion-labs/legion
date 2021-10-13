use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use legion_data_compiler::{
    compiler_api::{CompilationOutput, CompilerContext, CompilerError},
    CompilerHash, Locale, Platform, Target,
};
use legion_data_offline::resource::OfflineResource;
use legion_data_runtime::Resource;
use serde::Serialize;

use crate::offline_to_runtime::FromOffline;

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn compile<OfflineType, RuntimeType>(
    mut context: CompilerContext<'_>,
) -> Result<CompilationOutput, CompilerError>
where
    OfflineType: OfflineResource + 'static,
    RuntimeType: Resource + FromOffline<OfflineType> + Serialize,
{
    let mut resources = context.take_registry().add_loader::<OfflineType>().create();

    let resource = resources.load_sync::<OfflineType>(context.source.content_id());
    let resource = resource.get(&resources).unwrap();

    let asset = RuntimeType::from_offline(resource);
    let compiled_asset = bincode::serialize(&asset).unwrap();

    let asset = context.store(&compiled_asset, context.target_unnamed.clone())?;

    Ok(CompilationOutput {
        compiled_resources: vec![asset],
        resource_references: vec![],
    })
}
