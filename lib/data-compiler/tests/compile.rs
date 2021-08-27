use core::slice;
use std::fs::File;

use integer_asset::{IntegerAsset, IntegerAssetLoader};
use legion_content_store::{ContentStore, ContentStoreAddr, HddContentStore};
use legion_data_compiler::{
    compiler_cmd::{list_compilers, CompilerCompileCmd},
    Locale, Platform, Target,
};
use legion_data_offline::{
    asset::AssetPathId,
    resource::{ResourceId, ResourceProcessor, RESOURCE_EXT},
};
use legion_data_runtime::AssetLoader;
use text_resource::TextResource;

mod common;

#[test]
fn find_compiler() {
    let exe_path = common::compiler_exe("test-refs");
    assert!(exe_path.exists());

    let compilers = list_compilers(slice::from_ref(&common::target_dir()));
    assert_ne!(compilers.len(), 0);
}

#[test]
fn compile_atoi() {
    let work_dir = tempfile::tempdir().unwrap();
    let resource_dir = work_dir.path();

    let source_magic_value = String::from("47");

    let source = {
        let source = ResourceId::generate_new(text_resource::TYPE_ID);

        let mut proc = text_resource::TextResourceProc {};

        let mut resource = proc.new_resource();
        let mut resource = resource
            .as_any_mut()
            .downcast_mut::<TextResource>()
            .expect("valid resource");

        resource.content = source_magic_value.clone();

        let path = resource_dir.join(format!("{:x}.{}", source, RESOURCE_EXT));
        let mut file = File::create(path).expect("new file");
        proc.write_resource(resource, &mut file)
            .expect("written to disk");
        source
    };

    let cas_addr = ContentStoreAddr::from(resource_dir.to_owned());

    let asset_info = {
        let exe_path = common::compiler_exe("test-atoi");
        assert!(exe_path.exists());

        let compile_path = AssetPathId::from(source).push(integer_asset::TYPE_ID);
        let mut command = CompilerCompileCmd::new(
            &compile_path,
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
        result.compiled_resources[0].clone()
    };

    let checksum = asset_info.checksum;

    let cas = HddContentStore::open(cas_addr).expect("valid cas");
    assert!(cas.exists(checksum));

    let resource_content = cas.read(checksum).expect("asset content");

    let mut creator = IntegerAssetLoader {};
    let asset = creator
        .load(integer_asset::TYPE_ID, &mut &resource_content[..])
        .expect("loaded assets");
    let asset = asset.as_any().downcast_ref::<IntegerAsset>().unwrap();

    let stringified = asset.magic_value.to_string();
    assert_eq!(source_magic_value, stringified);
}

#[test]
fn compile_intermediate() {
    let work_dir = tempfile::tempdir().unwrap();
    let resource_dir = work_dir.path();

    let source_magic_value = String::from("47");

    let source = {
        let source = ResourceId::generate_new(text_resource::TYPE_ID);
        let mut proc = text_resource::TextResourceProc {};
        let mut resource = proc.new_resource();
        let mut resource = resource
            .as_any_mut()
            .downcast_mut::<TextResource>()
            .expect("valid resource");

        resource.content = source_magic_value.clone();

        let path = resource_dir.join(format!("{:x}.{}", source, RESOURCE_EXT));
        let mut file = File::create(path).expect("new file");
        proc.write_resource(resource, &mut file)
            .expect("written to disk");
        source
    };

    let cas_addr = ContentStoreAddr::from(resource_dir.to_owned());

    let intermediate_info = {
        let exe_path = common::compiler_exe("test-reverse");
        assert!(exe_path.exists());
        let compile_path = AssetPathId::from(source).push(text_resource::TYPE_ID);
        let mut command = CompilerCompileCmd::new(
            &compile_path,
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

        assert_eq!(result.compiled_resources.len(), 1);
        result.compiled_resources[0].clone()
    };

    let derived_info = {
        let exe_path = common::compiler_exe("test-atoi");
        assert!(exe_path.exists());
        let derived = AssetPathId::from(source)
            .push(text_resource::TYPE_ID)
            .push(integer_asset::TYPE_ID);
        let mut command = CompilerCompileCmd::new(
            &derived,
            &[],
            &[intermediate_info],
            &cas_addr,
            resource_dir,
            Target::Game,
            Platform::Windows,
            &Locale::new("en"),
        );

        let result = command
            .execute(&exe_path, resource_dir)
            .expect("compile result");

        assert_eq!(result.compiled_resources.len(), 1);
        result.compiled_resources[0].clone()
    };

    let checksum = derived_info.checksum;

    let cas = HddContentStore::open(cas_addr).expect("valid cas");
    assert!(cas.exists(checksum));

    let resource_content = cas.read(checksum).expect("asset content");

    let mut creator = IntegerAssetLoader {};
    let asset = creator
        .load(integer_asset::TYPE_ID, &mut &resource_content[..])
        .expect("loaded assets");
    let asset = asset.as_any().downcast_ref::<IntegerAsset>().unwrap();

    let stringified = asset.magic_value.to_string();
    assert_eq!(
        source_magic_value.chars().rev().collect::<String>(),
        stringified
    );
}

#[test]
fn compile_multi_resource() {
    let work_dir = tempfile::tempdir().unwrap();
    let resource_dir = work_dir.path();

    let source_text_list = vec![String::from("hello"), String::from("world")];

    let source = {
        let source = ResourceId::generate_new(multitext_resource::TYPE_ID);
        let mut proc = multitext_resource::MultiTextResourceProc {};
        let mut resource = proc.new_resource();
        let mut resource = resource
            .as_any_mut()
            .downcast_mut::<multitext_resource::MultiTextResource>()
            .expect("valid resource");

        resource.text_list = source_text_list.clone();

        let path = resource_dir.join(format!("{:x}.{}", source, RESOURCE_EXT));
        let mut file = File::create(path).expect("new file");
        proc.write_resource(resource, &mut file)
            .expect("written to disk");
        source
    };

    let cas_addr = ContentStoreAddr::from(resource_dir.to_owned());
    let compile_path = AssetPathId::from(source).push(text_resource::TYPE_ID);

    let compiled_resources = {
        let exe_path = common::compiler_exe("test-split");
        assert!(exe_path.exists());
        let compile_path = AssetPathId::from(source).push(text_resource::TYPE_ID);
        let mut command = CompilerCompileCmd::new(
            &compile_path,
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

        assert_eq!(result.compiled_resources.len(), source_text_list.len());
        result.compiled_resources
    };

    assert!(!(1..compiled_resources.len()).any(|i| {
        compiled_resources[i..]
            .iter()
            .any(|res| res.path == compiled_resources[i - 1].path)
    }));

    for (i, text_resource) in compiled_resources.iter().enumerate() {
        assert_eq!(
            text_resource.path,
            compile_path.new_named(&format!("text_{}", i))
        );
    }

    assert_eq!(compiled_resources.len(), source_text_list.len());
    let content_store = HddContentStore::open(cas_addr).expect("valid cas");

    for (resource, source_text) in compiled_resources.iter().zip(source_text_list.iter()) {
        assert!(content_store.exists(resource.checksum));
        let resource_content = content_store
            .read(resource.checksum)
            .expect("asset content");
        let mut proc = text_resource::TextResourceProc {};
        let resource = proc
            .read_resource(&mut &resource_content[..])
            .expect("loaded resource");
        let resource = resource.as_any().downcast_ref::<TextResource>().unwrap();
        assert_eq!(&resource.content, source_text);
    }
}
