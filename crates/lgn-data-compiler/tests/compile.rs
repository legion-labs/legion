use std::sync::Arc;

use binary_resource::BinaryResource;
use integer_asset::{IntegerAsset, IntegerAssetLoader};
use lgn_content_store::{
    indexing::{ResourceExists, ResourceReader},
    Config,
};
use lgn_data_compiler::compiler_cmd::{list_compilers, CompilerCompileCmd};
use lgn_data_runtime::{
    AssetLoader, ResourceDescriptor, ResourceId, ResourcePathId, ResourceProcessor,
    ResourceTypeAndId,
};
use multitext_resource::{MultiTextResource, MultiTextResourceProc};
use text_resource::TextResource;

mod common;

#[test]
fn find_compiler() {
    let exe_path = common::compiler_exe("test-refs");
    assert!(exe_path.exists());

    let compilers = list_compilers(std::slice::from_ref(&common::target_dir()));
    assert_ne!(compilers.len(), 0);
}

#[tokio::test]
async fn compile_atoi() {
    let persistent_content_provider = Arc::new(
        Config::load_and_instantiate_persistent_provider()
            .await
            .unwrap(),
    );

    let source_magic_value = String::from("47");

    let (source, source_manifest_id) = {
        let source = ResourceTypeAndId {
            kind: text_resource::TextResource::TYPE,
            id: ResourceId::new(),
        };

        let mut proc = text_resource::TextResourceProc {};

        let mut resource = proc.new_resource();
        let mut resource = resource
            .downcast_mut::<TextResource>()
            .expect("valid resource");

        resource.content = source_magic_value.clone();

        let source_manifest_id =
            common::write_resource(source, persistent_content_provider, &proc, resource).await;

        (source, source_manifest_id)
    };

    let asset_info = {
        let exe_path = common::compiler_exe("test-atoi");
        assert!(exe_path.exists());

        let compile_path = ResourcePathId::from(source).push(integer_asset::IntegerAsset::TYPE);
        let command = CompilerCompileCmd::new(
            &exe_path,
            &compile_path,
            &[],
            &[],
            &source_manifest_id,
            &common::test_env(),
        );

        let result = command.execute().await.expect("compile result");
        println!("{:?}", result);

        assert_eq!(result.compiled_resources.len(), 1);
        result.compiled_resources[0].clone()
    };

    let content_id = &asset_info.content_id;

    let volatile_content_provider = Config::load_and_instantiate_volatile_provider()
        .await
        .expect("failed to read config");

    assert!(volatile_content_provider
        .resource_exists(content_id)
        .await
        .unwrap());

    let resource_content = volatile_content_provider
        .read_resource_as_bytes(content_id)
        .await
        .expect("asset content");

    let mut loader = IntegerAssetLoader {};
    let asset = loader
        .load(&mut &resource_content[..])
        .expect("loaded assets");
    let asset = asset.downcast_ref::<IntegerAsset>().unwrap();

    let stringified = asset.magic_value.to_string();
    assert_eq!(source_magic_value, stringified);
}

#[tokio::test]
async fn compile_intermediate() {
    let persistent_content_provider = Arc::new(
        Config::load_and_instantiate_persistent_provider()
            .await
            .unwrap(),
    );

    let source_magic_value = String::from("47");

    let (source, source_manifest_id) = {
        let source = ResourceTypeAndId {
            kind: text_resource::TextResource::TYPE,
            id: ResourceId::new(),
        };
        let mut proc = text_resource::TextResourceProc {};
        let mut resource = proc.new_resource();
        let mut resource = resource
            .downcast_mut::<TextResource>()
            .expect("valid resource");

        resource.content = source_magic_value.clone();

        let source_manifest_id =
            common::write_resource(source, persistent_content_provider, &proc, resource).await;

        (source, source_manifest_id)
    };

    let intermediate_info = {
        let exe_path = common::compiler_exe("test-reverse");
        assert!(exe_path.exists());
        let compile_path = ResourcePathId::from(source).push(text_resource::TextResource::TYPE);
        let command = CompilerCompileCmd::new(
            &exe_path,
            &compile_path,
            &[],
            &[],
            &source_manifest_id,
            &common::test_env(),
        );

        let result = command.execute().await.expect("compile result");

        assert_eq!(result.compiled_resources.len(), 1);
        result.compiled_resources[0].clone()
    };

    let derived_info = {
        let exe_path = common::compiler_exe("test-atoi");
        assert!(exe_path.exists());
        let compile_path = ResourcePathId::from(source)
            .push(text_resource::TextResource::TYPE)
            .push(integer_asset::IntegerAsset::TYPE);
        let command = CompilerCompileCmd::new(
            &exe_path,
            &compile_path,
            &[],
            &[intermediate_info],
            &source_manifest_id,
            &common::test_env(),
        );

        let result = command.execute().await.expect("compile result");

        assert_eq!(result.compiled_resources.len(), 1);
        result.compiled_resources[0].clone()
    };

    let content_id = &derived_info.content_id;

    let volatile_content_provider = Config::load_and_instantiate_volatile_provider()
        .await
        .expect("failed to read config");

    assert!(volatile_content_provider
        .resource_exists(content_id)
        .await
        .unwrap());

    let resource_content = volatile_content_provider
        .read_resource_as_bytes(content_id)
        .await
        .expect("asset content");

    let mut loader = IntegerAssetLoader {};
    let asset = loader
        .load(&mut &resource_content[..])
        .expect("loaded assets");
    let asset = asset.downcast_ref::<IntegerAsset>().unwrap();

    let stringified = asset.magic_value.to_string();
    assert_eq!(
        source_magic_value.chars().rev().collect::<String>(),
        stringified
    );
}

#[tokio::test]
async fn compile_multi_resource() {
    let persistent_content_provider = Arc::new(
        Config::load_and_instantiate_persistent_provider()
            .await
            .unwrap(),
    );

    let source_text_list = vec![String::from("hello"), String::from("world")];

    let (source, source_manifest_id) = {
        let source = ResourceTypeAndId {
            kind: multitext_resource::MultiTextResource::TYPE,
            id: ResourceId::new(),
        };
        let mut proc = MultiTextResourceProc {};
        let mut resource = proc.new_resource();
        let mut resource = resource
            .downcast_mut::<MultiTextResource>()
            .expect("valid resource");

        resource.text_list = source_text_list.clone();

        let source_manifest_id =
            common::write_resource(source, persistent_content_provider, &proc, resource).await;

        (source, source_manifest_id)
    };

    let compile_path = ResourcePathId::from(source).push(text_resource::TextResource::TYPE);

    let compiled_resources = {
        let exe_path = common::compiler_exe("test-split");
        assert!(exe_path.exists());
        let compile_path = ResourcePathId::from(source).push(text_resource::TextResource::TYPE);
        let command = CompilerCompileCmd::new(
            &exe_path,
            &compile_path,
            &[],
            &[],
            &source_manifest_id,
            &common::test_env(),
        );

        let result = command.execute().await.expect("compile result");

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

    let volatile_content_provider = Config::load_and_instantiate_volatile_provider()
        .await
        .expect("failed to read config");

    for (resource, source_text) in compiled_resources.iter().zip(source_text_list.iter()) {
        assert!(volatile_content_provider
            .resource_exists(&resource.content_id)
            .await
            .unwrap());
        let resource_content = volatile_content_provider
            .read_resource_as_bytes(&resource.content_id)
            .await
            .expect("asset content");
        let mut proc = text_resource::TextResourceProc {};
        let resource = proc
            .read_resource(&mut &resource_content[..])
            .expect("loaded resource");
        let resource = resource.downcast_ref::<TextResource>().unwrap();
        assert_eq!(&resource.content, source_text);
    }
}

#[tokio::test]
async fn compile_base64() {
    let persistent_content_provider = Arc::new(
        Config::load_and_instantiate_persistent_provider()
            .await
            .unwrap(),
    );

    let source_binary_value = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    let expected_base64_value = String::from("AQIDBAUGBwgJ");

    let (source, source_manifest_id) = {
        let source = ResourceTypeAndId {
            kind: binary_resource::BinaryResource::TYPE,
            id: ResourceId::new(),
        };

        let mut proc = binary_resource::BinaryResourceProc {};

        let mut resource = proc.new_resource();
        let mut resource = resource
            .downcast_mut::<BinaryResource>()
            .expect("valid resource");

        resource.content = source_binary_value;

        let source_manifest_id =
            common::write_resource(source, persistent_content_provider, &proc, resource).await;

        (source, source_manifest_id)
    };

    let asset_info = {
        let exe_path = common::compiler_exe("test-base64");
        assert!(exe_path.exists());

        let compile_path = ResourcePathId::from(source).push(text_resource::TextResource::TYPE);
        let command = CompilerCompileCmd::new(
            &exe_path,
            &compile_path,
            &[],
            &[],
            &source_manifest_id,
            &common::test_env(),
        );

        let result = command.execute().await.expect("compile result");
        println!("{:?}", result);

        assert_eq!(result.compiled_resources.len(), 1);
        result.compiled_resources[0].clone()
    };

    let content_id = &asset_info.content_id;

    let volatile_content_provider = Config::load_and_instantiate_volatile_provider()
        .await
        .expect("failed to read config");

    assert!(volatile_content_provider
        .resource_exists(content_id)
        .await
        .unwrap());

    let resource_content = volatile_content_provider
        .read_resource_as_bytes(content_id)
        .await
        .expect("asset content");

    let base64str = String::from_utf8_lossy(&resource_content);
    assert_eq!(base64str, expected_base64_value);
}
