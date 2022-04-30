use std::{path::Path, sync::Arc};

use lgn_content_store::{Config, ContentReaderExt};
use lgn_data_compiler::compiler_cmd::{CompilerCompileCmd, CompilerHashCmd, CompilerInfoCmd};
use lgn_data_runtime::{
    AssetLoader, ResourceDescriptor, ResourceId, ResourcePathId, ResourceProcessor,
    ResourceTypeAndId,
};

mod common;

fn create_test_resource(id: ResourceTypeAndId, dir: &Path, content: &str) {
    let path = dir.join(id.id.resource_path());
    let mut file = common::create_resource_file(&path).expect("new file");

    let mut proc = refs_resource::TestResourceProc {};
    let mut resource = proc.new_resource();

    resource
        .downcast_mut::<refs_resource::TestResource>()
        .unwrap()
        .content = String::from(content);
    proc.write_resource(resource.as_ref(), &mut file)
        .expect("write to file");
}

#[test]
fn command_info() {
    let exe_path = common::compiler_exe("test-refs");
    assert!(exe_path.exists(), "{}", exe_path.display());

    let command = CompilerInfoCmd::new(&exe_path);
    let _info = command.execute().expect("info output");
}

#[test]
fn command_info_json() {
    let exe_path = common::compiler_exe("test-refs");
    assert!(exe_path.exists(), "{}", exe_path.display());

    let command = CompilerInfoCmd::new(&exe_path);
    let command_as_json = command.builder().to_string();

    let command = CompilerInfoCmd::from_slice(&command_as_json);
    let _info = command.execute().expect("info output");
}

#[test]
fn command_compiler_hash() {
    let exe_path = common::compiler_exe("test-refs");
    assert!(exe_path.exists(), "{}", exe_path.display());

    // get all hashes
    let command = CompilerHashCmd::new(&exe_path, &common::test_env(), None);
    let hashes = command.execute().expect("hash list");
    assert_eq!(hashes.compiler_hash_list.len(), 1);

    let (transform, hash) = hashes.compiler_hash_list[0];

    // get a hash for the specified transform
    let command = CompilerHashCmd::new(&exe_path, &common::test_env(), Some(transform));
    let hashes = command.execute().expect("hash list");
    assert_eq!(hashes.compiler_hash_list.len(), 1);
    assert_eq!(hashes.compiler_hash_list[0], (transform, hash));
}

#[tokio::test]
async fn command_compile() {
    let work_dir = tempfile::tempdir().unwrap();
    let (resource_dir, _output_dir) = common::setup_dir(&work_dir);

    let content = "test content";

    let source = ResourceTypeAndId {
        kind: refs_resource::TestResource::TYPE,
        id: ResourceId::new(),
    };
    create_test_resource(source, &resource_dir, content);

    let exe_path = common::compiler_exe("test-refs");
    assert!(exe_path.exists());

    let compile_path = ResourcePathId::from(source).push(refs_asset::RefsAsset::TYPE);
    let command = CompilerCompileCmd::new(
        &exe_path,
        &compile_path,
        &[],
        &[],
        &resource_dir,
        &common::test_env(),
    );

    let result = command.execute().expect("compile result");

    assert_eq!(result.compiled_resources.len(), 1);

    let content_id = &result.compiled_resources[0].content_id;

    let volatile_content_provider = Arc::new(
        Config::load_and_instantiate_volatile_provider()
            .await
            .unwrap(),
    );

    assert!(volatile_content_provider.exists(content_id).await);

    let resource_content = {
        let mut loader = refs_asset::RefsAssetLoader::default();
        let content = volatile_content_provider
            .read_content(content_id)
            .await
            .expect("asset content");
        let loaded_resource = loader.load(&mut &content[..]).expect("valid data");
        loaded_resource
            .as_ref()
            .downcast_ref::<refs_asset::RefsAsset>()
            .unwrap()
            .content
            .as_bytes()
            .to_owned()
    };
    let mut reversed = content.as_bytes().to_owned();
    reversed.reverse();
    assert_eq!(&resource_content[..], &reversed[..]);
}
