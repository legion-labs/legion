use std::{fs::File, path::Path};

use legion_content_store::{ContentStore, ContentStoreAddr, HddContentStore};
use legion_data_compiler::{
    compiler_cmd::{CompilerCompileCmd, CompilerHashCmd, CompilerInfoCmd},
    Locale, Platform, Target,
};
use legion_data_offline::{
    asset::AssetPathId,
    resource::{ResourceId, ResourceProcessor, RESOURCE_EXT},
};

mod common;

fn create_test_resource(id: ResourceId, dir: &Path, content: &str) {
    let path = dir.join(format!("{:x}.{}", id, RESOURCE_EXT));
    let mut file = File::create(path).expect("new file");

    let mut proc = refs_resource::TestResourceProc {};
    let mut resource = proc.new_resource();

    resource
        .as_any_mut()
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
    let resource_dir = work_dir.path();

    let content = "test content";

    let source = ResourceId::generate_new(refs_resource::TYPE_ID);
    create_test_resource(source, resource_dir, content);

    let exe_path = common::compiler_exe("test-refs");
    assert!(exe_path.exists());

    let cas_addr = ContentStoreAddr::from(resource_dir.to_owned());

    let derived = AssetPathId::from(source).transform(refs_resource::TYPE_ID);
    let mut command = CompilerCompileCmd::new(
        &derived,
        &[],
        &[],
        &cas_addr,
        resource_dir,
        Target::Game,
        Platform::Windows,
        &Locale::new("en"),
    );

    let result = command
        .execute(&exe_path, resource_dir)
        .expect("compile result");
    println!("{:?}", result);

    assert_eq!(result.compiled_resources.len(), 1);

    let checksum = result.compiled_resources[0].checksum;

    let cas = HddContentStore::open(cas_addr).expect("valid cas");
    assert!(cas.exists(checksum));

    let resource_content = cas.read(checksum).expect("asset content");
    let mut reversed = content.as_bytes().to_owned();
    reversed.reverse();
    assert_eq!(&resource_content[..], &reversed[..]);
}
