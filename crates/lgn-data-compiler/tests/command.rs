use std::path::Path;

use lgn_content_store::{ContentStore, ContentStoreAddr, HddContentStore};
use lgn_data_compiler::compiler_cmd::{CompilerCompileCmd, CompilerHashCmd, CompilerInfoCmd};
use lgn_data_offline::{resource::ResourceProcessor, ResourcePathId};
use lgn_data_runtime::{AssetLoader, Resource, ResourceId, ResourceTypeAndId};

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
    assert!(exe_path.exists());

    let command = CompilerInfoCmd::default();
    let _info = command.execute(&exe_path).expect("info output");
}

#[test]
fn command_compiler_hash() {
    let exe_path = common::compiler_exe("test-refs");
    assert!(exe_path.exists(), "{}", exe_path.display());

    // get all hashes
    let command = CompilerHashCmd::new(&common::test_env(), None);
    let hashes = command.execute(&exe_path).expect("hash list");
    assert_eq!(hashes.compiler_hash_list.len(), 1);

    let (transform, hash) = hashes.compiler_hash_list[0];

    // get a hash for the specified transform
    let command = CompilerHashCmd::new(&common::test_env(), Some(transform));
    let hashes = command.execute(&exe_path).expect("hash list");
    assert_eq!(hashes.compiler_hash_list.len(), 1);
    assert_eq!(hashes.compiler_hash_list[0], (transform, hash));
}

#[test]
fn command_compile() {
    let work_dir = tempfile::tempdir().unwrap();
    let (resource_dir, output_dir) = common::setup_dir(&work_dir);

    let content = "test content";

    let source = ResourceTypeAndId {
        kind: refs_resource::TestResource::TYPE,
        id: ResourceId::new(),
    };
    create_test_resource(source, &resource_dir, content);

    let exe_path = common::compiler_exe("test-refs");
    assert!(exe_path.exists());

    let cas_addr = ContentStoreAddr::from(output_dir);

    let compile_path = ResourcePathId::from(source).push(refs_asset::RefsAsset::TYPE);
    let mut command = CompilerCompileCmd::new(
        &compile_path,
        &[],
        &[],
        &cas_addr,
        &resource_dir,
        &common::test_env(),
    );

    let result = command.execute(&exe_path).expect("compile result");

    assert_eq!(result.compiled_resources.len(), 1);

    let checksum = result.compiled_resources[0].checksum;

    let cas = HddContentStore::open(cas_addr).expect("valid cas");
    assert!(cas.exists(checksum));

    let resource_content = {
        let mut loader = refs_asset::RefsAssetLoader::default();
        let content = cas.read(checksum).expect("asset content");
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
