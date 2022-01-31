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
        let mut project = Project::create_new(&project_dir)
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
        DataBuildOptions::new(&output_dir, CompilerRegistryOptions::default())
            .content_store(&ContentStoreAddr::from(output_dir))
            .create_with_project(project_dir)
            .await
            .expect("data build");

    let updated_count = build.source_pull(&project).await.unwrap();
    assert_eq!(updated_count, 1);

    let updated_count = build.source_pull(&project).await.unwrap();
    assert_eq!(updated_count, 0);

    assert!(build
        .build_index
        .source_index
        .find_dependencies(&resource)
        .is_some());
    assert_eq!(
        build
            .build_index
            .source_index
            .find_dependencies(&resource)
            .unwrap()
            .len(),
        0
    );
}

#[tokio::test]
async fn with_dependency() {
    let work_dir = tempfile::tempdir().unwrap();
    let (project_dir, output_dir) = setup_dir(&work_dir);
    let resources = setup_registry();
    let mut resources = resources.lock().await;

    let (child_id, parent_id) = {
        let mut project = Project::create_new(&project_dir)
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
        DataBuildOptions::new(&output_dir, CompilerRegistryOptions::default())
            .content_store(&ContentStoreAddr::from(output_dir))
            .create_with_project(project_dir)
            .await
            .expect("data build");

    let updated_count = build.source_pull(&project).await.unwrap();
    assert_eq!(updated_count, 2);

    let child_deps = build
        .build_index
        .source_index
        .find_dependencies(&child_id)
        .expect("zero deps");
    let parent_deps = build
        .build_index
        .source_index
        .find_dependencies(&parent_id)
        .expect("one dep");

    assert_eq!(child_deps.len(), 0);
    assert_eq!(parent_deps.len(), 1);

    let updated_count = build.source_pull(&project).await.unwrap();
    assert_eq!(updated_count, 0);
}

#[tokio::test]
async fn with_derived_dependency() {
    let work_dir = tempfile::tempdir().unwrap();
    let (project_dir, output_dir) = setup_dir(&work_dir);
    let resources = setup_registry();
    let mut resources = resources.lock().await;

    {
        let mut project = Project::create_new(&project_dir)
            .await
            .expect("failed to create a project");

        let child_id = project
            .add_resource(
                ResourcePathName::new("intermediate_child"),
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
            let intermediate_id =
                ResourcePathId::from(child_id).push(refs_resource::TestResource::TYPE);

            let res = resources
                .new_resource(refs_resource::TestResource::TYPE)
                .unwrap();
            res.get_mut::<refs_resource::TestResource>(&mut resources)
                .unwrap()
                .build_deps
                .push(intermediate_id);
            res
        };
        let _parent_id = project
            .add_resource(
                ResourcePathName::new("intermetidate_parent"),
                refs_resource::TestResource::TYPENAME,
                refs_resource::TestResource::TYPE,
                &parent_handle,
                &mut resources,
            )
            .await
            .unwrap();
    }

    let (mut build, project) =
        DataBuildOptions::new(&output_dir, CompilerRegistryOptions::default())
            .content_store(&ContentStoreAddr::from(output_dir))
            .create_with_project(project_dir)
            .await
            .expect("to create index");

    let updated_count = build.source_pull(&project).await.unwrap();
    assert_eq!(updated_count, 3);

    let updated_count = build.source_pull(&project).await.unwrap();
    assert_eq!(updated_count, 0);
}
