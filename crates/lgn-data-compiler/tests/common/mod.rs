use std::{env, path::PathBuf};

use lgn_content_store::{
    indexing::{
        BasicIndexer, ResourceByteWriter, ResourceWriter, Tree, TreeIdentifier, TreeLeafNode,
        TreeWriter,
    },
    Provider,
};
use lgn_data_compiler::{compiler_api::CompilationEnv, Locale, Platform, Target};
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
    provider: &Provider,
    proc: &impl ResourceProcessor,
    resource: &dyn Resource,
) -> TreeIdentifier {
    let mut bytes = std::io::Cursor::new(Vec::new());

    proc.write_resource(resource, &mut bytes)
        .expect("write to memory");

    let resource_id = provider
        .write_resource(&ResourceByteWriter::new(&bytes.into_inner()))
        .await
        .expect("write to content-store");

    let indexer = new_resource_type_and_id_indexer();
    let offline_manifest_id = provider
        .write_tree(&Tree::default())
        .await
        .expect("initialize content-store manifest");
    indexer
        .add_leaf(
            provider,
            &offline_manifest_id,
            &id.into(),
            TreeLeafNode::Resource(resource_id),
        )
        .await
        .expect("write manifest to content-store")
}
