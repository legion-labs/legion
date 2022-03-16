use std::path::PathBuf;
use std::sync::Arc;

use lgn_content_store::ContentStoreAddr;
use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
use lgn_data_offline::{
    resource::{Project, ResourcePathName, ResourceRegistry, ResourceRegistryOptions},
    ResourcePathId,
};
use lgn_data_runtime::Resource;
use tempfile::TempDir;

use crate::DataBuildOptions;

fn setup_registry() -> Arc<tokio::sync::Mutex<ResourceRegistry>> {
    ResourceRegistryOptions::new()
        .add_type::<refs_resource::TestResource>()
        .create_async_registry()
}

fn setup_dir(work_dir: &TempDir) -> (PathBuf, PathBuf) {
    let project_dir = work_dir.path();
    let output_dir = project_dir.join("temp");
    std::fs::create_dir(&output_dir).unwrap();
    (project_dir.to_owned(), output_dir)
}

#[tokio::test]
async fn no_dependencies() {
    let work_dir = tempfile::tempdir().unwrap();
    let (project_dir, output_dir) = setup_dir(&work_dir);
    let resources = setup_registry();
    let mut resources = resources.lock().await;

    let resource = {
        let mut project = Project::create_with_remote_mock(&project_dir)
            .await
            .expect("failed to create a project");
        let id = project
            .add_resource(
                ResourcePathName::new("resource"),
                refs_resource::TestResource::TYPENAME,
                refs_resource::TestResource::TYPE,
                &resources
                    .new_resource(refs_resource::TestResource::TYPE)
                    .unwrap(),
                &mut resources,
            )
            .await
            .unwrap();
        ResourcePathId::from(id)
    };

    let (mut build, project) =
        DataBuildOptions::new_with_sqlite_output(&output_dir, CompilerRegistryOptions::default())
            .content_store(&ContentStoreAddr::from(output_dir))
            .create_with_project(project_dir)
            .await
            .expect("data build");

    build.source_pull(&project).await.unwrap();

    let source_index = build.source_index.current().unwrap();

    assert!(source_index.find_dependencies(&resource).is_some());
    assert_eq!(source_index.find_dependencies(&resource).unwrap().len(), 0);
}

#[tokio::test]
async fn with_dependency() {
    let work_dir = tempfile::tempdir().unwrap();
    let (project_dir, output_dir) = setup_dir(&work_dir);
    let resources = setup_registry();
    let mut resources = resources.lock().await;

    let (child_id, parent_id) = {
        let mut project = Project::create_with_remote_mock(&project_dir)
            .await
            .expect("failed to create a project");
        let child_id = project
            .add_resource(
                ResourcePathName::new("child"),
                refs_resource::TestResource::TYPENAME,
                refs_resource::TestResource::TYPE,
                &resources
                    .new_resource(refs_resource::TestResource::TYPE)
                    .unwrap(),
                &mut resources,
            )
            .await
            .unwrap();

        let parent_handle = {
            let res = resources
                .new_resource(refs_resource::TestResource::TYPE)
                .unwrap();
            res.get_mut::<refs_resource::TestResource>(&mut resources)
                .unwrap()
                .build_deps
                .push(ResourcePathId::from(child_id));
            res
        };
        let parent_id = project
            .add_resource(
                ResourcePathName::new("parent"),
                refs_resource::TestResource::TYPENAME,
                refs_resource::TestResource::TYPE,
                &parent_handle,
                &mut resources,
            )
            .await
            .unwrap();
        (
            ResourcePathId::from(child_id),
            ResourcePathId::from(parent_id),
        )
    };

    let (mut build, project) =
        DataBuildOptions::new_with_sqlite_output(&output_dir, CompilerRegistryOptions::default())
            .content_store(&ContentStoreAddr::from(output_dir))
            .create_with_project(project_dir)
            .await
            .expect("data build");

    build.source_pull(&project).await.unwrap();

    let source_index = build.source_index.current().unwrap();

    let child_deps = source_index
        .find_dependencies(&child_id)
        .expect("zero deps");
    let parent_deps = source_index.find_dependencies(&parent_id).expect("one dep");

    assert_eq!(child_deps.len(), 0);
    assert_eq!(parent_deps.len(), 1);
}
