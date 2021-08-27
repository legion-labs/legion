use crate::DataBuildOptions;
use legion_content_store::ContentStoreAddr;
use legion_data_offline::{
    asset::AssetPathId,
    resource::{Project, ResourceName, ResourceRegistry},
};

pub const TEST_BUILDINDEX_FILENAME: &str = "build.index";

fn setup_registry() -> ResourceRegistry {
    let mut resources = ResourceRegistry::default();
    resources.register_type(
        refs_resource::TYPE_ID,
        Box::new(refs_resource::TestResourceProc {}),
    );
    resources.register_type(
        refs_resource::TYPE_ID,
        Box::new(refs_resource::TestResourceProc {}),
    );
    resources
}

#[test]
fn no_dependencies() {
    let work_dir = tempfile::tempdir().unwrap();
    let project_dir = work_dir.path();
    let mut resources = setup_registry();

    let resource = {
        let mut project = Project::create_new(project_dir).expect("failed to create a project");
        let id = project
            .add_resource(
                ResourceName::from("resource"),
                refs_resource::TYPE_ID,
                &resources.new_resource(refs_resource::TYPE_ID).unwrap(),
                &mut resources,
            )
            .unwrap();
        AssetPathId::from(id)
    };

    let mut build = DataBuildOptions::new(project_dir.join(TEST_BUILDINDEX_FILENAME))
        .content_store(&ContentStoreAddr::from(project_dir.to_owned()))
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
    let project_dir = work_dir.path();
    let mut resources = setup_registry();

    let (child_id, parent_id) = {
        let mut project = Project::create_new(project_dir).expect("failed to create a project");
        let child_id = project
            .add_resource(
                ResourceName::from("child"),
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
                ResourceName::from("parent"),
                refs_resource::TYPE_ID,
                &parent_handle,
                &mut resources,
            )
            .unwrap();
        (AssetPathId::from(child_id), AssetPathId::from(parent_id))
    };

    let mut build = DataBuildOptions::new(project_dir.join(TEST_BUILDINDEX_FILENAME))
        .content_store(&ContentStoreAddr::from(project_dir.to_owned()))
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
    let project_dir = work_dir.path();
    let mut resources = setup_registry();

    {
        let mut project = Project::create_new(project_dir).expect("failed to create a project");

        let child_id = project
            .add_resource(
                ResourceName::from("intermediate_child"),
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
                ResourceName::from("intermetidate_parent"),
                refs_resource::TYPE_ID,
                &parent_handle,
                &mut resources,
            )
            .unwrap();
    }

    let mut build = DataBuildOptions::new(project_dir.join(TEST_BUILDINDEX_FILENAME))
        .content_store(&ContentStoreAddr::from(project_dir.to_owned()))
        .create(project_dir)
        .expect("to create index");

    let updated_count = build.source_pull().unwrap();
    assert_eq!(updated_count, 3);

    let updated_count = build.source_pull().unwrap();
    assert_eq!(updated_count, 0);
}
