use std::{env, path::PathBuf, sync::Arc};

use lgn_content_store::{
    indexing::{ResourceIndex, ResourceWriter, TreeIdentifier},
    Provider,
};
use lgn_data_compiler::{compiler_api::CompilationEnv, Locale, Platform, Target};
use lgn_data_offline::resource::{serialize_metadata, ResourcePathName};
use lgn_data_runtime::{
    new_resource_type_and_id_indexer, Resource, ResourceProcessor, ResourceTypeAndId,
};

pub fn target_dir() -> PathBuf {
    env::current_exe()
        .ok()
        .map(|mut path| {
            path.pop();
            if path.ends_with("deps") {
                path.pop();
            }
            path
        })
        .expect("available test directory")
}

pub fn compiler_exe(name: &str) -> PathBuf {
    target_dir().join(format!("compiler-{}{}", name, env::consts::EXE_SUFFIX))
}

pub fn test_env() -> CompilationEnv {
    CompilationEnv {
        target: Target::Game,
        platform: Platform::Windows,
        locale: Locale::new("en"),
    }
}

pub async fn write_resource(
    id: ResourceTypeAndId,
    provider: Arc<Provider>,
    proc: &impl ResourceProcessor,
    resource: &dyn Resource,
) -> TreeIdentifier {
    let mut bytes = std::io::Cursor::new(Vec::new());

    // pre-pend metadata before serialized resource
    let name = ResourcePathName::new("test_resource");
    let dependencies = proc.extract_build_dependencies(resource);
    serialize_metadata(name, id, dependencies, &mut bytes);

    proc.write_resource(resource, &mut bytes)
        .expect("write to memory");

    let resource_id = provider
        .write_resource_from_bytes(&bytes.into_inner())
        .await
        .expect("write to content-store");

    let mut source_manifest =
        ResourceIndex::new_exclusive(provider, new_resource_type_and_id_indexer()).await;
    source_manifest
        .add_resource(&id.into(), resource_id)
        .await
        .expect("write manifest to content-store");
    source_manifest.id()
}
