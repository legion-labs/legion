use std::sync::Arc;

use generic_data::offline::{BinaryResource, IntegerAsset, MultiTextResource, TextResource};
use lgn_content_store::{indexing::ResourceExists, Config};
use lgn_data_compiler::compiler_cmd::{list_compilers, CompilerCompileCmd};
use lgn_data_offline::{from_json_reader, SourceResource};
use lgn_data_runtime::prelude::*;

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
            kind: TextResource::TYPE,
            id: ResourceId::new(),
        };
        let mut resource = TextResource::new_named("test_resource");
        resource.content = source_magic_value.clone();

        let source_manifest_id =
            common::write_resource(source, Arc::clone(&persistent_content_provider), &resource)
                .await;

        (source, source_manifest_id)
    };

    let asset_info = {
        let exe_path = common::compiler_exe("test-atoi");
        assert!(exe_path.exists());

        let compile_path = ResourcePathId::from(source).push(IntegerAsset::TYPE);
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

    let reader = volatile_content_provider
        .get_reader(content_id.as_identifier())
        .await
        .expect("asset content");

    IntegerAsset::register_resource_type();
    let mut reader = Box::pin(reader) as AssetRegistryReader;
    let asset = from_json_reader::<IntegerAsset>(&mut reader)
        .await
        .expect("loaded assets");

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
            kind: TextResource::TYPE,
            id: ResourceId::new(),
        };
        let mut resource = TextResource::new_with_id("test_resource", source);
        resource.content = source_magic_value.clone();

        let source_manifest_id =
            common::write_resource(source, Arc::clone(&persistent_content_provider), &resource)
                .await;

        (source, source_manifest_id)
    };

    let intermediate_info = {
        let exe_path = common::compiler_exe("test-reverse");
        assert!(exe_path.exists());
        let compile_path = ResourcePathId::from(source).push(TextResource::TYPE);
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
            .push(TextResource::TYPE)
            .push(IntegerAsset::TYPE);
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

    let reader = volatile_content_provider
        .get_reader(content_id.as_identifier())
        .await
        .expect("asset content");

    IntegerAsset::register_resource_type();
    let mut reader = Box::pin(reader) as AssetRegistryReader;
    let asset = from_json_reader::<IntegerAsset>(&mut reader)
        .await
        .expect("loaded assets");

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
            kind: MultiTextResource::TYPE,
            id: ResourceId::new(),
        };
        let mut resource = MultiTextResource::new_named("test_resource");
        resource.text_list = source_text_list.clone();

        let source_manifest_id =
            common::write_resource(source, Arc::clone(&persistent_content_provider), &resource)
                .await;

        (source, source_manifest_id)
    };

    let compile_path = ResourcePathId::from(source).push(TextResource::TYPE);

    let compiled_resources = {
        let exe_path = common::compiler_exe("test-split");
        assert!(exe_path.exists());
        let compile_path = ResourcePathId::from(source).push(TextResource::TYPE);
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
        let reader = volatile_content_provider
            .get_reader(resource.content_id.as_identifier())
            .await
            .expect("asset content");

        TextResource::register_resource_type();
        let mut reader = Box::pin(reader) as AssetRegistryReader;
        let resource = from_json_reader::<TextResource>(&mut reader)
            .await
            .expect("loaded resource");

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
            kind: BinaryResource::TYPE,
            id: ResourceId::new(),
        };

        let mut resource = BinaryResource::new_named("test_resource");
        resource.content = source_binary_value;

        let source_manifest_id =
            common::write_resource(source, Arc::clone(&persistent_content_provider), &resource)
                .await;

        (source, source_manifest_id)
    };

    let asset_info = {
        let exe_path = common::compiler_exe("test-base64");
        assert!(exe_path.exists());

        let compile_path = ResourcePathId::from(source).push(TextResource::TYPE);
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

    let reader = volatile_content_provider
        .get_reader(content_id.as_identifier())
        .await
        .expect("asset content");

    TextResource::register_resource_type();
    let mut reader = Box::pin(reader) as AssetRegistryReader;
    let text_resource = from_json_reader::<TextResource>(&mut reader).await.unwrap();

    let base64str = text_resource.content;
    assert_eq!(base64str, expected_base64_value);
}
