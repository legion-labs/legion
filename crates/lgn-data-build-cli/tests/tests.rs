use std::fs;

use lgn_content_store::{ContentStoreAddr, HddContentStore};
use lgn_data_build::{DataBuild, DataBuildOptions};
use lgn_data_compiler::{
    compiler_api::CompilationEnv, compiler_node::CompilerRegistryOptions, Locale, Platform, Target,
};
use lgn_data_offline::{
    resource::{Project, ResourcePathName, ResourceRegistryOptions},
    ResourcePathId,
};
use lgn_data_runtime::{AssetRegistryOptions, Resource};

static DATABUILD_EXE: &str = env!("CARGO_BIN_EXE_data-build");

#[tokio::test]
async fn build_device() {
    let work_dir = tempfile::tempdir().unwrap();

    let cas = work_dir.path().join("content_store");
    let project_dir = work_dir.path();
    let output_dir = work_dir.path();

    // create output directory
    fs::create_dir(&cas).expect("new directory");

    let initial_content = "foo";

    // create project that contains test resource.
    let source_id = {
        let mut project = Project::create_with_remote_mock(project_dir)
            .await
            .expect("new project");
        let resources = ResourceRegistryOptions::new()
            .add_type::<refs_resource::TestResource>()
            .create_async_registry();
        let mut resources = resources.lock().await;

        let resource = resources
            .new_resource(refs_resource::TestResource::TYPE)
            .expect("new resource")
            .typed::<refs_resource::TestResource>();

        resource.get_mut(&mut resources).unwrap().content = initial_content.to_string();

        project
            .add_resource(
                ResourcePathName::new("test_source"),
                refs_resource::TestResource::TYPENAME,
                refs_resource::TestResource::TYPE,
                &resource,
                &mut resources,
            )
            .await
            .expect("adding the resource")
    };

    let target_dir = {
        std::env::current_exe().ok().map_or_else(
            || panic!("cannot find test directory"),
            |mut path| {
                path.pop();
                if path.ends_with("deps") {
                    path.pop();
                }
                path
            },
        )
    };

    // create build index.
    let (mut build, project) = DataBuildOptions::new_with_sqlite_output(
        &output_dir,
        CompilerRegistryOptions::local_compilers(target_dir),
    )
    .content_store(&ContentStoreAddr::from(cas.clone()))
    .create_with_project(project_dir)
    .await
    .expect("new build index");
    build.source_pull(&project).await.expect("successful pull");

    // the transformation below will reverse source resource's content.
    let derived = ResourcePathId::from(source_id).push(refs_asset::RefsAsset::TYPE);
    let derived_content = initial_content.chars().rev().collect::<String>();

    // build derived resource first, so that buildindex is aware of the
    // ResourcePathId
    build
        .compile(
            derived.clone(),
            &CompilationEnv {
                target: Target::Game,
                platform: Platform::Windows,
                locale: Locale::new("en"),
            },
        )
        .await
        .expect("successful compilation");

    assert_eq!(
        build
            .lookup_pathid(derived.resource_id())
            .await
            .unwrap()
            .as_ref(),
        Some(&derived)
    );

    // create resource registry that uses the 'build device'
    let cas_addr = ContentStoreAddr::from(cas);
    let content_store = HddContentStore::open(cas_addr.clone()).expect("valid cas");
    let manifest = lgn_data_runtime::manifest::Manifest::default();
    let registry = AssetRegistryOptions::new()
        .add_loader::<refs_resource::TestResource>()
        .add_loader::<refs_asset::RefsAsset>()
        .add_device_build(
            Box::new(content_store),
            cas_addr,
            manifest,
            DATABUILD_EXE,
            DataBuildOptions::output_db_path_dir(output_dir, project_dir, DataBuild::version()),
            project_dir,
            true,
        )
        .create()
        .await;

    // build needs to be dropped to flush recorded ResourcePathIds to disk
    std::mem::drop(build);

    // load (and build/fetch from cache) derived resource
    let derived_id = derived.resource_id();
    {
        let handle = registry
            .load_async::<refs_asset::RefsAsset>(derived_id)
            .await;
        assert!(handle.is_loaded(&registry));

        let resource = handle.get(&registry).expect("loaded asset");
        assert_eq!(resource.content, derived_content);
    }

    // change content
    let changed_content = "bar";
    let changed_derived_content = changed_content.chars().rev().collect::<String>();
    {
        let mut project = Project::open(project_dir).await.expect("new project");
        let resources = ResourceRegistryOptions::new()
            .add_type::<refs_resource::TestResource>()
            .create_async_registry();
        let mut resources = resources.lock().await;

        let resource = project
            .load_resource(source_id, &mut resources)
            .expect("existing resource")
            .typed::<refs_resource::TestResource>();

        let res = resource.get_mut(&mut resources).expect("loaded resource");
        res.content = changed_content.to_string();

        project
            .save_resource(source_id, resource, &mut resources)
            .await
            .expect("successful save");
    }

    registry.update();

    // load (and recompile) the changed resource
    let handle = registry
        .load_async::<refs_asset::RefsAsset>(derived_id)
        .await;
    assert!(handle.is_loaded(&registry));

    let resource = handle.get(&registry).expect("loaded asset");
    assert_eq!(resource.content, changed_derived_content);
}

#[tokio::test]
async fn no_intermediate_resource() {
    let work_dir = tempfile::tempdir().unwrap();

    let cas = work_dir.path().join("content_store");
    let project_dir = work_dir.path();
    let output_dir = work_dir.path();

    // create output directory
    fs::create_dir(&cas).expect("new directory");

    // create project that contains test resource.
    let resource_id = {
        let resource_id = {
            let mut project = Project::create_with_remote_mock(project_dir)
                .await
                .expect("new project");
            let resources = ResourceRegistryOptions::new()
                .add_type::<refs_resource::TestResource>()
                .create_async_registry();
            let mut resources = resources.lock().await;

            let resource = resources
                .new_resource(refs_resource::TestResource::TYPE)
                .expect("new resource");

            project
                .add_resource(
                    ResourcePathName::new("test_source"),
                    refs_resource::TestResource::TYPENAME,
                    refs_resource::TestResource::TYPE,
                    &resource,
                    &mut resources,
                )
                .await
                .expect("adding the resource")
        };
        let (mut build, project) = DataBuildOptions::new(
            DataBuildOptions::output_db_path_dir(output_dir, &project_dir, DataBuild::version()),
            ContentStoreAddr::from(cas.clone()),
            CompilerRegistryOptions::default(),
        )
        .create_with_project(project_dir)
        .await
        .expect("new build index");
        build.source_pull(&project).await.expect("successful pull");

        resource_id
    };

    let compile_path = ResourcePathId::from(resource_id).push(refs_asset::RefsAsset::TYPE);

    let mut command = {
        let target = "game";
        let platform = "windows";
        let locale = "en";
        let mut command = std::process::Command::new(DATABUILD_EXE);
        command.arg("compile");
        command.arg(compile_path.to_string());
        command.arg(format!("--cas={}", cas.to_str().unwrap()));
        command.arg(format!("--target={}", target));
        command.arg(format!("--platform={}", platform));
        command.arg(format!("--locale={}", locale));
        command.arg(format!("--output={}", output_dir.to_str().unwrap()));
        command.arg(format!("--project={}", project_dir.to_str().unwrap()));
        command
    };

    let output = command.output().expect("valid output");
    if !output.status.success() {
        println!(
            "{:?}",
            std::str::from_utf8(&output.stdout).expect("valid utf8")
        );
        println!(
            "{:?}",
            std::str::from_utf8(&output.stderr).expect("valid utf8")
        );
    }

    assert!(output.status.success());
    let _manifest: lgn_data_compiler::CompiledResources =
        serde_json::from_slice(&output.stdout).expect("valid manifest");
}

#[tokio::test]
async fn with_intermediate_resource() {
    let work_dir = tempfile::tempdir().unwrap();

    let cas = work_dir.path().join("content_store");
    let project_dir = work_dir.path();
    let output_dir = work_dir.path();

    // create output directory
    fs::create_dir(&cas).expect("new directory");

    // create project that contains test resource.
    let resource_id = {
        let resource_id = {
            let mut project = Project::create_with_remote_mock(project_dir)
                .await
                .expect("new project");
            let resources = ResourceRegistryOptions::new()
                .add_type::<text_resource::TextResource>()
                .create_async_registry();
            let mut resources = resources.lock().await;

            let resource = resources
                .new_resource(text_resource::TextResource::TYPE)
                .expect("new resource");

            project
                .add_resource(
                    ResourcePathName::new("test_source"),
                    text_resource::TextResource::TYPENAME,
                    text_resource::TextResource::TYPE,
                    &resource,
                    &mut resources,
                )
                .await
                .expect("adding the resource")
        };
        let (mut build, project) = DataBuildOptions::new_with_sqlite_output(
            &output_dir,
            CompilerRegistryOptions::default(),
        )
        .content_store(&ContentStoreAddr::from(cas.clone()))
        .create_with_project(project_dir)
        .await
        .expect("new build index");
        build.source_pull(&project).await.expect("successful pull");

        resource_id
    };

    let compile_path = ResourcePathId::from(resource_id)
        .push(text_resource::TextResource::TYPE)
        .push(integer_asset::IntegerAsset::TYPE);

    let mut command = {
        let target = "game";
        let platform = "windows";
        let locale = "en";
        let mut command = std::process::Command::new(DATABUILD_EXE);
        command.arg("compile");
        command.arg(compile_path.to_string());
        command.arg(format!("--cas={}", cas.to_str().unwrap()));
        command.arg(format!("--target={}", target));
        command.arg(format!("--platform={}", platform));
        command.arg(format!("--locale={}", locale));
        command.arg(format!("--output={}", output_dir.to_str().unwrap()));
        command.arg(format!("--project={}", project_dir.to_str().unwrap()));
        command
    };

    let output = command.output().expect("valid output");
    if !output.status.success() {
        println!(
            "{:?}",
            std::str::from_utf8(&output.stdout).expect("valid utf8")
        );
        println!(
            "{:?}",
            std::str::from_utf8(&output.stderr).expect("valid utf8")
        );
    }

    assert!(output.status.success());
    let _manifest: lgn_data_compiler::CompiledResources =
        serde_json::from_slice(&output.stdout).expect("valid manifest");
}
