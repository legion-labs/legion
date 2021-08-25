use std::fs;

use legion_content_store::ContentStoreAddr;
use legion_data_build::{DataBuildOptions, ResourceName};
use legion_resources::{Project, ResourcePathId, ResourceRegistry};

static DATABUILD_EXE: &str = env!("CARGO_BIN_EXE_data-build");

#[test]
fn no_intermediate_resource() {
    let work_dir = tempfile::tempdir().unwrap();

    let cas = work_dir.path().join("content_store");
    let project_dir = work_dir.path();
    let buildindex_path = work_dir.path().join("build.index");
    let manifest_path = work_dir.path().join("output.manifest");

    // create output directory
    fs::create_dir(&cas).expect("new directory");

    // create project that contains test resource.
    let resource_id = {
        let resource_id = {
            let mut project = Project::create_new(project_dir).expect("new project");
            let mut resources = ResourceRegistry::default();
            resources.register_type(
                refs_resource::TYPE_ID,
                Box::new(refs_resource::TestResourceProc {}),
            );
            let resource = resources
                .new_resource(refs_resource::TYPE_ID)
                .expect("new resource");

            project
                .add_resource(
                    ResourceName::from("test_source"),
                    refs_resource::TYPE_ID,
                    &resource,
                    &mut resources,
                )
                .expect("adding the resource")
        };
        let mut build = DataBuildOptions::new(&buildindex_path)
            .content_store(&ContentStoreAddr::from(cas.clone()))
            .create(project_dir)
            .expect("new build index");
        build.source_pull().expect("successful pull");

        resource_id
    };

    let compile_path = ResourcePathId::from(resource_id).transform(refs_resource::TYPE_ID);

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
        command.arg(format!(
            "--buildindex={}",
            buildindex_path.to_str().unwrap()
        ));
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

    let output = std::str::from_utf8(&output.stdout).expect("valid utf8");
    assert!(output.contains("CompiledResource"));
}

#[test]
fn with_intermediate_resource() {
    let work_dir = tempfile::tempdir().unwrap();

    let cas = work_dir.path().join("content_store");
    let project_dir = work_dir.path();
    let buildindex_path = work_dir.path().join("build.index");
    let manifest_path = work_dir.path().join("output.manifest");

    // create output directory
    fs::create_dir(&cas).expect("new directory");

    // create project that contains test resource.
    let resource_id = {
        let resource_id = {
            let mut project = Project::create_new(project_dir).expect("new project");
            let mut resources = ResourceRegistry::default();
            resources.register_type(
                text_resource::TEXT_RESOURCE,
                Box::new(text_resource::TextResourceProc {}),
            );
            let resource = resources
                .new_resource(text_resource::TEXT_RESOURCE)
                .expect("new resource");

            project
                .add_resource(
                    ResourceName::from("test_source"),
                    text_resource::TEXT_RESOURCE,
                    &resource,
                    &mut resources,
                )
                .expect("adding the resource")
        };
        let mut build = DataBuildOptions::new(&buildindex_path)
            .content_store(&ContentStoreAddr::from(cas.clone()))
            .create(project_dir)
            .expect("new build index");
        build.source_pull().expect("successful pull");

        resource_id
    };

    let compile_path = ResourcePathId::from(resource_id)
        .transform(text_resource::TEXT_RESOURCE)
        .transform(integer_asset::INTEGER_ASSET);

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
        command.arg(format!(
            "--buildindex={}",
            buildindex_path.to_str().unwrap()
        ));
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

    let output = std::str::from_utf8(&output.stdout).expect("valid utf8");
    assert!(output.contains("CompiledResource"));
}
