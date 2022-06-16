use std::{env, path::PathBuf, sync::Arc};

use lgn_content_store::{
    indexing::{ResourceIndex, ResourceWriter, TreeIdentifier},
    Provider,
};
use lgn_data_compiler::{compiler_api::CompilationEnv, Locale, Platform, Target};
use lgn_data_runtime::{new_resource_type_and_id_indexer, Resource, ResourceTypeAndId};

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
    persistent_provider: Arc<Provider>,
    resource: &dyn Resource,
) -> TreeIdentifier {
    let mut bytes = Vec::<u8>::new();
    lgn_data_offline::to_json_writer(resource, &mut bytes).unwrap();

    let resource_id = persistent_provider
        .write_resource_from_bytes(&bytes)
        .await
        .expect("write to content-store");

    let mut source_manifest =
        ResourceIndex::new_exclusive(persistent_provider, new_resource_type_and_id_indexer()).await;
    source_manifest
        .add_resource(&id.into(), resource_id)
        .await
        .expect("write manifest to content-store");
    source_manifest.id()
}
