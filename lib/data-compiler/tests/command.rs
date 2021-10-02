use std::{fs::File, path::Path};

use legion_content_store::{ContentStore, ContentStoreAddr, HddContentStore};
use legion_data_compiler::{
    compiler_cmd::{CompilerCompileCmd, CompilerHashCmd, CompilerInfoCmd},
    Locale, Platform, Target,
};
use legion_data_offline::{resource::ResourceProcessor, ResourcePathId};
use legion_data_runtime::{AssetDescriptor, ResourceId};

mod common;

fn create_test_resource(id: ResourceId, dir: &Path, content: &str) {
    let path = dir.join(format!("{:x}", id));
    let mut file = File::create(path).expect("new file");

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
    assert!(exe_path.exists());

    let command = CompilerHashCmd::new(Target::Game, Platform::Windows, &Locale::new("en"));
    let _hashes = command.execute(&exe_path).expect("hash list");
}

#[test]
fn command_compile() {
    let work_dir = tempfile::tempdir().unwrap();
    let (resource_dir, output_dir) = common::setup_dir(&work_dir);

    let content = "test content";

    let source = ResourceId::new_random_id(refs_resource::TYPE_ID);
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
        Target::Game,
        Platform::Windows,
        &Locale::new("en"),
    );

    let result = command.execute(&exe_path).expect("compile result");

    assert_eq!(result.compiled_resources.len(), 1);

    let checksum = result.compiled_resources[0].checksum;

    let cas = HddContentStore::open(cas_addr).expect("valid cas");
    assert!(cas.exists(checksum.get()));

    let resource_content = cas.read(checksum.get()).expect("asset content");
    let mut reversed = content.as_bytes().to_owned();
    reversed.reverse();
    assert_eq!(&resource_content[..], &reversed[..]);
}
