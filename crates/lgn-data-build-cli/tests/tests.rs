use std::{fs, sync::Arc};

use generic_data::{
    offline::RefsAsset,
    offline::{IntegerAsset, TestResource, TextResource},
};
use lgn_data_build::{DataBuild, DataBuildOptions};
use lgn_data_compiler::{
    compiler_api::CompilationEnv, compiler_node::CompilerRegistryOptions, Locale, Platform, Target,
};
use lgn_data_offline::{Project, SourceResource};
use lgn_data_runtime::{AssetRegistryOptions, ResourceDescriptor, ResourcePathId};
use lgn_source_control::{RepositoryIndex, RepositoryName};
use serial_test::serial;

static DATABUILD_EXE: &str = env!("CARGO_BIN_EXE_data-build");

#[tokio::test]
#[serial]
async fn build_device() {
    let work_dir = tempfile::tempdir().unwrap();
    std::env::set_var("WORK_DIR", work_dir.path().to_str().unwrap());

    let legion_toml = include_str!("legion.toml");
    fs::write(work_dir.path().join("legion.toml"), legion_toml).unwrap();
    std::env::set_var(
        "LGN_CONFIG",
        work_dir.path().join("legion.toml").to_str().unwrap(),
    );

    let project_dir = work_dir.path();
    let output_dir = work_dir.path();
    let repository_index = lgn_source_control::Config::load_and_instantiate_repository_index()
        .await
        .unwrap();
    let repository_name: RepositoryName = "default".parse().unwrap();
    let branch_name = "main";
    repository_index
        .create_repository(&repository_name)
        .await
        .unwrap();
    let source_control_content_provider = Arc::new(
        lgn_content_store::Config::load_and_instantiate_persistent_provider()
            .await
            .unwrap(),
    );
    let data_content_provider = Arc::new(
        lgn_content_store::Config::load_and_instantiate_volatile_provider()
            .await
            .unwrap(),
    );

    let initial_content = "foo";

    // create project that contains test resource.
    let mut project = Project::new(
        &repository_index,
        &repository_name,
        branch_name,
        Arc::clone(&source_control_content_provider),
    )
    .await
    .expect("new project");

    let source_id = {

        let mut resource = TestResource::new_named("test_source");
        resource.content = initial_content.to_string();

        let source_id = project
            .add_resource(&resource)
            .await
            .expect("adding the resource");

        project.commit("add resource").await.expect("committing");

        source_id
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
    let mut build = DataBuildOptions::new_with_sqlite_output(
        &output_dir,
        CompilerRegistryOptions::local_compilers(target_dir),
        Arc::clone(&source_control_content_provider),
        Arc::clone(&data_content_provider),
    )
    .create(&project)
    .await
    .expect("new build index");
    build.source_pull(&project).await.expect("successful pull");

    // the transformation below will reverse source resource's content.
    let derived = ResourcePathId::from(source_id).push(RefsAsset::TYPE);
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

    let manifest = lgn_data_runtime::manifest::Manifest::default();
    let mut options = AssetRegistryOptions::new().add_device_build(
            Arc::clone(&data_content_provider),
            None,
            DATABUILD_EXE,
            &DataBuildOptions::output_db_path_dir(output_dir, project_dir, DataBuild::version()),
            repository_name.as_str(),
            branch_name,
            true,
        )
        .await;

    generic_data::register_types(&mut options);
    let registry = options.create().await;

    // build needs to be dropped to flush recorded ResourcePathIds to disk
    std::mem::drop(build);

    // load (and build/fetch from cache) derived resource
    let derived_id = derived.resource_id();
    {
        let handle = registry
            .load_async::<RefsAsset>(derived_id)
            .await
            .expect("loaded asset");

        let resource = handle.get().expect("loaded asset");
        assert_eq!(resource.content, derived_content);
    }

    // change content
    let changed_content = "bar";
    let changed_derived_content = changed_content.chars().rev().collect::<String>();
    {

        let mut resource = project
            .load_resource::<TestResource>(source_id.id)
            .await
            .expect("existing resource");

        resource.content = changed_content.to_string();

        project
            .save_resource(source_id.id, resource.as_ref())
            .await
            .expect("successful save");

        project.commit("save resource").await.expect("committing");
    }

    registry.update();

    // load (and recompile) the changed resource
    let handle = registry.load_async::<RefsAsset>(derived_id).await.unwrap();

    let resource = handle.get().expect("loaded asset");
    assert_eq!(resource.content, changed_derived_content);
}

#[tokio::test]
#[serial]
async fn no_intermediate_resource() {
    let work_dir = tempfile::tempdir().unwrap();
    std::env::set_var("WORK_DIR", work_dir.path().to_str().unwrap());

    let legion_toml = include_str!("legion.toml");
    fs::write(work_dir.path().join("legion.toml"), legion_toml).unwrap();
    std::env::set_var(
        "LGN_CONFIG",
        work_dir.path().join("legion.toml").to_str().unwrap(),
    );

    let project_dir = work_dir.path();
    let output_dir = work_dir.path();
    let repository_index = lgn_source_control::Config::load_and_instantiate_repository_index()
        .await
        .unwrap();
    let repository_name: RepositoryName = "default".parse().unwrap();
    let branch_name = "main";
    repository_index
        .create_repository(&repository_name)
        .await
        .unwrap();
    let source_control_content_provider = Arc::new(
        lgn_content_store::Config::load_and_instantiate_persistent_provider()
            .await
            .unwrap(),
    );
    let data_content_provider = Arc::new(
        lgn_content_store::Config::load_and_instantiate_volatile_provider()
            .await
            .unwrap(),
    );

    // create project that contains test resource.
    let resource_id = {
        let mut project = Project::new(
            &repository_index,
            &repository_name,
            branch_name,
            Arc::clone(&source_control_content_provider),
        )
        .await
        .expect("new project");

        let resource_id = {

            let resource = TestResource::new_named("test_source");
            let resource_id = project
                .add_resource(&resource)
                .await
                .expect("adding the resource");

            project.commit("add resource").await.expect("committing");

            resource_id
        };

        let mut build = DataBuildOptions::new(
            DataBuildOptions::output_db_path_dir(output_dir, &project_dir, DataBuild::version()),
            Arc::clone(&source_control_content_provider),
            Arc::clone(&data_content_provider),
            CompilerRegistryOptions::default(),
        )
        .create(&project)
        .await
        .expect("new build index");
        build.source_pull(&project).await.expect("successful pull");

        resource_id
    };

    let compile_path = ResourcePathId::from(resource_id).push(RefsAsset::TYPE);

    let mut command = {
        let target = "game";
        let platform = "windows";
        let locale = "en";
        let mut command = std::process::Command::new(DATABUILD_EXE);
        command.arg("compile");
        command.arg(compile_path.to_string());
        command.arg(format!("--target={}", target));
        command.arg(format!("--platform={}", platform));
        command.arg(format!("--locale={}", locale));
        command.arg(format!("--output={}", output_dir.to_str().unwrap()));
        command.arg(format!("--repository-name={}", repository_name));
        command.arg(format!("--branch-name={}", branch_name));
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
#[serial]
async fn with_intermediate_resource() {
    let work_dir = tempfile::tempdir().unwrap();
    std::env::set_var("WORK_DIR", work_dir.path().to_str().unwrap());

    let legion_toml = include_str!("legion.toml");
    fs::write(work_dir.path().join("legion.toml"), legion_toml).unwrap();
    std::env::set_var(
        "LGN_CONFIG",
        work_dir.path().join("legion.toml").to_str().unwrap(),
    );

    let output_dir = work_dir.path();
    let repository_index = lgn_source_control::Config::load_and_instantiate_repository_index()
        .await
        .unwrap();
    let repository_name: RepositoryName = "default".parse().unwrap();
    let branch_name = "main";
    repository_index
        .create_repository(&repository_name)
        .await
        .unwrap();

    let source_control_content_provider = Arc::new(
        lgn_content_store::Config::load_and_instantiate_persistent_provider()
            .await
            .unwrap(),
    );
    let data_content_provider = Arc::new(
        lgn_content_store::Config::load_and_instantiate_volatile_provider()
            .await
            .unwrap(),
    );

    // create project that contains test resource.
    let resource_id = {
        let mut project = Project::new(
            &repository_index,
            &repository_name,
            branch_name,
            Arc::clone(&source_control_content_provider),
        )
        .await
        .expect("new project");

        let resource_id = {

            let resource = TextResource::new_named("test_source");

            let resource_id = project
                .add_resource(&resource)
                .await
                .expect("adding the resource");

            project.commit("add resource").await.expect("committing");

            resource_id
        };

        let mut build = DataBuildOptions::new_with_sqlite_output(
            &output_dir,
            CompilerRegistryOptions::default(),
            Arc::clone(&source_control_content_provider),
            Arc::clone(&data_content_provider),
        )
        .create(&project)
        .await
        .expect("new build index");
        build.source_pull(&project).await.expect("successful pull");

        resource_id
    };

    let compile_path = ResourcePathId::from(resource_id)
        .push(TextResource::TYPE)
        .push(IntegerAsset::TYPE);

    let mut command = {
        let target = "game";
        let platform = "windows";
        let locale = "en";
        let mut command = std::process::Command::new(DATABUILD_EXE);
        //command.env("LGN_CONFIG", legion_toml);
        command.arg("compile");
        command.arg(compile_path.to_string());
        command.arg(format!("--target={}", target));
        command.arg(format!("--platform={}", platform));
        command.arg(format!("--locale={}", locale));
        command.arg(format!("--output={}", output_dir.to_str().unwrap()));
        command.arg(format!("--repository-name={}", repository_name));
        command.arg(format!("--branch-name={}", branch_name));
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
