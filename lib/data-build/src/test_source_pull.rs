use std::path::PathBuf;

use crate::DataBuildOptions;
use legion_content_store::ContentStoreAddr;
use legion_data_offline::{
    asset::AssetPathId,
    resource::{Project, ResourcePathName, ResourceRegistry, ResourceRegistryOptions},
};
use tempfile::TempDir;

pub const TEST_BUILDINDEX_FILENAME: &str = "build.index";

fn setup_registry() -> ResourceRegistry {
    ResourceRegistryOptions::new()
        .add_type(
            refs_resource::TYPE_ID,
            Box::new(refs_resource::TestResourceProc {}),
        )
        .create_registry()
}

fn setup_dir(work_dir: &TempDir) -> (PathBuf, PathBuf) {
    let project_dir = work_dir.path();
    let output_dir = project_dir.join("temp");
    std::fs::create_dir(&output_dir).unwrap();
    (project_dir.to_owned(), output_dir)
}

#[test]
fn no_dependencies() {
    let work_dir = tempfile::tempdir().unwrap();
    let (project_dir, output_dir) = setup_dir(&work_dir);
    let mut resources = setup_registry();

    let resource = {
        let mut project = Project::create_new(&project_dir).expect("failed to create a project");
        let id = project
            .add_resource(
                ResourcePathName::new("resource"),
                refs_resource::TYPE_ID,
                &resources.new_resource(refs_resource::TYPE_ID).unwrap(),
                &mut resources,
            )
            .unwrap();
        AssetPathId::from(id)
    };

    let mut build = DataBuildOptions::new(output_dir.join(TEST_BUILDINDEX_FILENAME))
        .content_store(&ContentStoreAddr::from(output_dir))
        .create(project_dir)
        .expect("data build");

    let updated_count = build.source_pull().unwrap();
    assert_eq!(updated_count, 1);

    let updated_count = build.source_pull().unwrap();
    assert_eq!(updated_count, 0);

    assert!(build.build_index.find_dependencies(&resource).is_some());
    assert_eq!(
        build
            .build_index
            .find_dependencies(&resource)
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn with_dependency() {
    let work_dir = tempfile::tempdir().unwrap();
    let (project_dir, output_dir) = setup_dir(&work_dir);
    let mut resources = setup_registry();

    let (child_id, parent_id) = {
        let mut project = Project::create_new(&project_dir).expect("failed to create a project");
        let child_id = project
            .add_resource(
                ResourcePathName::new("child"),
                refs_resource::TYPE_ID,
                &resources.new_resource(refs_resource::TYPE_ID).unwrap(),
                &mut resources,
            )
            .unwrap();

        let parent_handle = {
            let res = resources.new_resource(refs_resource::TYPE_ID).unwrap();
            res.get_mut::<refs_resource::TestResource>(&mut resources)
                .unwrap()
                .build_deps
                .push(AssetPathId::from(child_id));
            res
        };
        let parent_id = project
            .add_resource(
                ResourcePathName::new("parent"),
                refs_resource::TYPE_ID,
                &parent_handle,
                &mut resources,
            )
            .unwrap();
        (AssetPathId::from(child_id), AssetPathId::from(parent_id))
    };

    let mut build = DataBuildOptions::new(output_dir.join(TEST_BUILDINDEX_FILENAME))
        .content_store(&ContentStoreAddr::from(output_dir))
        .create(project_dir)
        .expect("data build");

    let updated_count = build.source_pull().unwrap();
    assert_eq!(updated_count, 2);

    let child_deps = build
        .build_index
        .find_dependencies(&child_id)
        .expect("zero deps");
    let parent_deps = build
        .build_index
        .find_dependencies(&parent_id)
        .expect("one dep");

    assert_eq!(child_deps.len(), 0);
    assert_eq!(parent_deps.len(), 1);

    let updated_count = build.source_pull().unwrap();
    assert_eq!(updated_count, 0);
}

#[test]
fn with_derived_dependency() {
    let work_dir = tempfile::tempdir().unwrap();
    let (project_dir, output_dir) = setup_dir(&work_dir);
    let mut resources = setup_registry();

    {
        let mut project = Project::create_new(&project_dir).expect("failed to create a project");

        let child_id = project
            .add_resource(
                ResourcePathName::new("intermediate_child"),
                refs_resource::TYPE_ID,
                &resources.new_resource(refs_resource::TYPE_ID).unwrap(),
                &mut resources,
            )
            .unwrap();

        let parent_handle = {
            let intermediate_id = AssetPathId::from(child_id).push(refs_resource::TYPE_ID);

            let res = resources.new_resource(refs_resource::TYPE_ID).unwrap();
            res.get_mut::<refs_resource::TestResource>(&mut resources)
                .unwrap()
                .build_deps
                .push(intermediate_id);
            res
        };
        let _parent_id = project
            .add_resource(
                ResourcePathName::new("intermetidate_parent"),
                refs_resource::TYPE_ID,
                &parent_handle,
                &mut resources,
            )
            .unwrap();
    }

    let mut build = DataBuildOptions::new(output_dir.join(TEST_BUILDINDEX_FILENAME))
        .content_store(&ContentStoreAddr::from(output_dir))
        .create(project_dir)
        .expect("to create index");

    let updated_count = build.source_pull().unwrap();
    assert_eq!(updated_count, 3);

    let updated_count = build.source_pull().unwrap();
    assert_eq!(updated_count, 0);
}
