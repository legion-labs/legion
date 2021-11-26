use std::fs;

use legion_content_store::{ContentStoreAddr, HddContentStore};
use legion_data_build::DataBuildOptions;
use legion_data_compiler::{Locale, Platform, Target};
use legion_data_offline::{
    resource::{Project, ResourcePathName, ResourceRegistryOptions},
    ResourcePathId,
};
use legion_data_runtime::{AssetRegistryOptions, Resource};

static DATABUILD_EXE: &str = env!("CARGO_BIN_EXE_data-build");

#[test]
fn build_device() {
    let work_dir = tempfile::tempdir().unwrap();

    let cas = work_dir.path().join("content_store");
    let project_dir = work_dir.path();
    let buildindex_dir = work_dir.path();

    // create output directory
    fs::create_dir(&cas).expect("new directory");

    let initial_content = "foo";

    // create project that contains test resource.
    let source_id = {
        let mut project = Project::create_new(project_dir).expect("new project");
        let resources = ResourceRegistryOptions::new()
            .add_type::<refs_resource::TestResource>()
            .create_registry();
        let mut resources = resources.lock().unwrap();

        let resource = resources
            .new_resource(refs_resource::TestResource::TYPE)
            .expect("new resource")
            .typed::<refs_resource::TestResource>();

        resource.get_mut(&mut resources).unwrap().content = initial_content.to_string();

        project
            .add_resource(
                ResourcePathName::new("test_source"),
                refs_resource::TestResource::TYPE,
                &resource,
                &mut resources,
            )
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
    let mut build = DataBuildOptions::new(&buildindex_dir)
        .content_store(&ContentStoreAddr::from(cas.clone()))
        .compiler_dir(target_dir)
        .create(project_dir)
        .expect("new build index");
    build.source_pull().expect("successful pull");

    // the transformation below will reverse source resource's content.
    let derived = ResourcePathId::from(source_id).push(refs_asset::RefsAsset::TYPE);
    let derived_content = initial_content.chars().rev().collect::<String>();

    // build derived resource first, so that buildindex is aware of the ResourcePathId
    build
        .compile(
            derived.clone(),
            None,
            Target::Game,
            Platform::Windows,
            &Locale::new("en"),
        )
        .expect("successful compilation");

    // create resource registry that uses the 'build device'
    let cas_addr = ContentStoreAddr::from(cas);
    let content_store = HddContentStore::open(cas_addr.clone()).expect("valid cas");
    let manifest = legion_data_runtime::manifest::Manifest::default();
    let registry = AssetRegistryOptions::new()
        .add_loader::<refs_resource::TestResource>()
        .add_loader::<refs_asset::RefsAsset>()
        .add_device_build(
            Box::new(content_store),
            cas_addr,
            manifest,
            DATABUILD_EXE,
            buildindex_dir,
            true,
        )
        .create();

    // build needs to be dropped to flush recorded ResourcePathIds to disk
    std::mem::drop(build);

    // load (and build/fetch from cache) derived resource
    let derived_id = derived.resource_id();
    {
        let handle = registry.load_sync::<refs_asset::RefsAsset>(derived_id);
        assert!(handle.is_loaded(&registry));

        let resource = handle.get(&registry).expect("loaded asset");
        assert_eq!(resource.content, derived_content);
    }

    // change content
    let changed_content = "bar";
    let changed_derived_content = changed_content.chars().rev().collect::<String>();
    {
        let mut project = Project::open(project_dir).expect("new project");
        let resources = ResourceRegistryOptions::new()
            .add_type::<refs_resource::TestResource>()
            .create_registry();
        let mut resources = resources.lock().unwrap();

        let resource = project
            .load_resource(source_id, &mut resources)
            .expect("existing resource")
            .typed::<refs_resource::TestResource>();

        let res = resource.get_mut(&mut resources).expect("loaded resource");
        res.content = changed_content.to_string();

        project
            .save_resource(source_id, resource, &mut resources)
            .expect("successful save");
    }

    registry.update();

    // load (and recompile) the changed resource
    let handle = registry.load_sync::<refs_asset::RefsAsset>(derived_id);
    assert!(handle.is_loaded(&registry));

    let resource = handle.get(&registry).expect("loaded asset");
    assert_eq!(resource.content, changed_derived_content);
}

#[test]
fn no_intermediate_resource() {
    let work_dir = tempfile::tempdir().unwrap();

    let cas = work_dir.path().join("content_store");
    let project_dir = work_dir.path();
    let buildindex_dir = work_dir.path();
    let manifest_path = work_dir.path().join("output.manifest");

    // create output directory
    fs::create_dir(&cas).expect("new directory");

    // create project that contains test resource.
    let resource_id = {
        let resource_id = {
            let mut project = Project::create_new(project_dir).expect("new project");
            let resources = ResourceRegistryOptions::new()
                .add_type::<refs_resource::TestResource>()
                .create_registry();
            let mut resources = resources.lock().unwrap();

            let resource = resources
                .new_resource(refs_resource::TestResource::TYPE)
                .expect("new resource");

            project
                .add_resource(
                    ResourcePathName::new("test_source"),
                    refs_resource::TestResource::TYPE,
                    &resource,
                    &mut resources,
                )
                .expect("adding the resource")
        };
        let mut build = DataBuildOptions::new(&buildindex_dir)
            .content_store(&ContentStoreAddr::from(cas.clone()))
            .create(project_dir)
            .expect("new build index");
        build.source_pull().expect("successful pull");

        resource_id
    };

    let compile_path = ResourcePathId::from(resource_id).push(refs_asset::RefsAsset::TYPE);

    let mut command = {
        let target = "game";
        let platform = "windows";
        let locale = "en";
        let mut command = std::process::Command::new(DATABUILD_EXE);
        command.arg("compile");
        command.arg(format!("{}", compile_path));
        command.arg(format!("--cas={}", cas.to_str().unwrap()));
        command.arg(format!("--target={}", target));
        command.arg(format!("--platform={}", platform));
        command.arg(format!("--locale={}", locale));
        command.arg(format!(
            "--manifest={}",
            manifest_path.to_str().expect("valid path")
        ));
        command.arg(format!("--buildindex={}", buildindex_dir.to_str().unwrap()));
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
    let _manifest: legion_data_compiler::Manifest =
        serde_json::from_slice(&output.stdout).expect("valid manifest");
}

#[test]
fn with_intermediate_resource() {
    let work_dir = tempfile::tempdir().unwrap();

    let cas = work_dir.path().join("content_store");
    let project_dir = work_dir.path();
    let buildindex_dir = work_dir.path();
    let manifest_path = work_dir.path().join("output.manifest");

    // create output directory
    fs::create_dir(&cas).expect("new directory");

    // create project that contains test resource.
    let resource_id = {
        let resource_id = {
            let mut project = Project::create_new(project_dir).expect("new project");
            let resources = ResourceRegistryOptions::new()
                .add_type::<text_resource::TextResource>()
                .create_registry();
            let mut resources = resources.lock().unwrap();

            let resource = resources
                .new_resource(text_resource::TextResource::TYPE)
                .expect("new resource");

            project
                .add_resource(
                    ResourcePathName::new("test_source"),
                    text_resource::TextResource::TYPE,
                    &resource,
                    &mut resources,
                )
                .expect("adding the resource")
        };
        let mut build = DataBuildOptions::new(&buildindex_dir)
            .content_store(&ContentStoreAddr::from(cas.clone()))
            .create(project_dir)
            .expect("new build index");
        build.source_pull().expect("successful pull");

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
        command.arg(format!("{}", compile_path));
        command.arg(format!("--cas={}", cas.to_str().unwrap()));
        command.arg(format!("--target={}", target));
        command.arg(format!("--platform={}", platform));
        command.arg(format!("--locale={}", locale));
        command.arg(format!(
            "--manifest={}",
            manifest_path.to_str().expect("valid path")
        ));
        command.arg(format!("--buildindex={}", buildindex_dir.to_str().unwrap()));
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
    let _manifest: legion_data_compiler::Manifest =
        serde_json::from_slice(&output.stdout).expect("valid manifest");
}
