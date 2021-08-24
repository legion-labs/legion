use std::fs::File;
use std::path::{Path, PathBuf};
use std::{env, vec};

use crate::databuild::CompileOutput;
use crate::{databuild::DataBuild, DataBuildOptions};
use legion_content_store::{ContentStore, ContentStoreAddr, HddContentStore};
use legion_data_compiler::{Locale, Manifest, Platform, Target};
use legion_resources::{Project, ResourceId, ResourceName, ResourcePathId, ResourceRegistry};

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

fn target_dir() -> PathBuf {
    env::current_exe().ok().map_or_else(
        || panic!("cannot find test directory"),
        |mut path| {
            path.pop();
            if path.ends_with("deps") {
                path.pop();
            }
            path
        },
    )
}

fn create_resource(
    name: ResourceName,
    deps: &[ResourcePathId],
    project: &mut Project,
    resources: &mut ResourceRegistry,
) -> ResourceId {
    let resource_b = {
        let res = resources.new_resource(refs_resource::TYPE_ID).unwrap();
        let resource = res
            .get_mut::<refs_resource::TestResource>(resources)
            .unwrap();
        resource.content = name.display().to_string(); // each resource needs unique content to generate a unique resource.
        resource.build_deps.extend_from_slice(deps);
        res
    };
    project
        .add_resource(name, refs_resource::TYPE_ID, &resource_b, resources)
        .unwrap()
}

fn change_resource(resource_id: ResourceId, project_dir: &Path) {
    let mut project = Project::open(project_dir).expect("failed to open project");
    let mut resources = setup_registry();

    let handle = project
        .load_resource(resource_id, &mut resources)
        .expect("to load resource");

    let resource = handle
        .get_mut::<refs_resource::TestResource>(&mut resources)
        .expect("resource instance");
    resource.content.push_str(" more content");
    project
        .save_resource(resource_id, &handle, &mut resources)
        .expect("successful save");
}

#[test]
fn compile_change_no_deps() {
    let work_dir = tempfile::tempdir().unwrap();
    let project_dir = work_dir.path();
    let mut resources = setup_registry();

    let (resource_id, resource_handle) = {
        let mut project = Project::create_new(project_dir).expect("failed to create a project");

        let resource_handle = resources.new_resource(refs_resource::TYPE_ID).unwrap();
        let resource_id = project
            .add_resource(
                ResourceName::from("resource"),
                refs_resource::TYPE_ID,
                &resource_handle,
                &mut resources,
            )
            .unwrap();
        (resource_id, resource_handle)
    };

    let contentstore_path = ContentStoreAddr::from(work_dir.path());
    let mut config = DataBuildOptions::new(project_dir.join(TEST_BUILDINDEX_FILENAME));
    config
        .content_store(&contentstore_path)
        .compiler_dir(target_dir());

    let target = ResourcePathId::from(resource_id).transform(refs_resource::TYPE_ID);

    // compile the resource..
    let original_checksum = {
        let mut build = config.create(project_dir).expect("to create index");
        build.source_pull().expect("failed to pull from project");

        let compile_output = build
            .compile_path(
                target.clone(),
                Target::Game,
                Platform::Windows,
                &Locale::new("en"),
            )
            .unwrap();

        assert_eq!(compile_output.resources.len(), 1);
        assert_eq!(compile_output.references.len(), 0);

        let original_checksum = compile_output.resources[0].compiled_checksum;

        let content_store =
            HddContentStore::open(contentstore_path.clone()).expect("valid content store");
        assert!(content_store.exists(original_checksum));

        original_checksum
    };

    // ..change resource..
    {
        let mut project = Project::open(project_dir).expect("failed to open project");

        resource_handle
            .get_mut::<refs_resource::TestResource>(&mut resources)
            .unwrap()
            .content = String::from("new content");

        project
            .save_resource(resource_id, &resource_handle, &mut resources)
            .unwrap();
    }

    // ..re-compile changed resource..
    let modified_checksum = {
        let mut build = config.open().expect("to open index");
        build.source_pull().expect("failed to pull from project");
        let compile_output = build
            .compile_path(target, Target::Game, Platform::Windows, &Locale::new("en"))
            .unwrap();

        assert_eq!(compile_output.resources.len(), 1);

        let modified_checksum = compile_output.resources[0].compiled_checksum;

        let content_store = HddContentStore::open(contentstore_path).expect("valid content store");
        assert!(content_store.exists(original_checksum));
        assert!(content_store.exists(modified_checksum));

        modified_checksum
    };

    assert_ne!(original_checksum, modified_checksum);
}

/// Creates a project with 5 resources with dependencies setup as depicted below.
/// t(A) depicts a dependency on a `derived resource A` transformed  by `t`.
/// Returns an array of resources from A to E where A is at index 0.
//
// t(A) -> A -> t(B) -> B -> t(C) -> C
//         |            |
//         V            |
//       t(D)           |
//         |            |
//         V            V
//         D -------> t(E) -> E
//
fn setup_project(project_dir: impl AsRef<Path>) -> [ResourceId; 5] {
    let mut project =
        Project::create_new(project_dir.as_ref()).expect("failed to create a project");

    let mut resources = setup_registry();

    let res_c = create_resource(ResourceName::from("C"), &[], &mut project, &mut resources);
    let res_e = create_resource(ResourceName::from("E"), &[], &mut project, &mut resources);
    let res_d = create_resource(
        ResourceName::from("D"),
        &[ResourcePathId::from(res_e).transform(refs_resource::TYPE_ID)],
        &mut project,
        &mut resources,
    );
    let res_b = create_resource(
        ResourceName::from("B"),
        &[
            ResourcePathId::from(res_c).transform(refs_resource::TYPE_ID),
            ResourcePathId::from(res_e).transform(refs_resource::TYPE_ID),
        ],
        &mut project,
        &mut resources,
    );
    let res_a = create_resource(
        ResourceName::from("A"),
        &[
            ResourcePathId::from(res_b).transform(refs_resource::TYPE_ID),
            ResourcePathId::from(res_d).transform(refs_resource::TYPE_ID),
        ],
        &mut project,
        &mut resources,
    );
    [res_a, res_b, res_c, res_d, res_e]
}

#[test]
fn compile_cache() {
    let work_dir = tempfile::tempdir().unwrap();
    let project_dir = work_dir.path();

    let resource_list = setup_project(project_dir);
    let root_resource = resource_list[0];

    let mut build = DataBuildOptions::new(project_dir.join(TEST_BUILDINDEX_FILENAME))
        .content_store(&ContentStoreAddr::from(work_dir.path()))
        .compiler_dir(target_dir())
        .create(project_dir)
        .expect("new build index");
    build.source_pull().expect("successful pull");

    //
    // test(A) -> A -> test(B) -> B -> test(C) -> C
    //            |               |
    //            V               |
    //          test(D)           |
    //            |               |
    //            V               V
    //            D ---------> test(E) -> E
    //
    const NUM_NODES: usize = 10;
    const NUM_OUTPUTS: usize = 5;
    let target = ResourcePathId::from(root_resource).transform(refs_resource::TYPE_ID);

    //  test of evaluation order computation.
    {
        let order = build
            .build_index
            .evaluation_order(target.clone())
            .expect("no cycles");
        assert_eq!(order.len(), NUM_NODES);
        assert_eq!(order[NUM_NODES - 1], target);
        assert_eq!(order[NUM_NODES - 2], ResourcePathId::from(root_resource));
    }

    // first run - none of the resources from cache.
    {
        let CompileOutput {
            resources,
            references,
            statistics,
        } = build
            .compile_path(
                target.clone(),
                Target::Game,
                Platform::Windows,
                &Locale::new("en"),
            )
            .expect("successful compilation");

        assert_eq!(resources.len(), NUM_OUTPUTS);
        assert_eq!(references.len(), NUM_OUTPUTS);
        assert!(statistics.iter().all(|s| !s.from_cache));
    }

    // no change, second run - all resources from cache.
    {
        let CompileOutput {
            resources,
            references,
            statistics,
        } = build
            .compile_path(
                target.clone(),
                Target::Game,
                Platform::Windows,
                &Locale::new("en"),
            )
            .expect("successful compilation");

        assert_eq!(resources.len(), NUM_OUTPUTS);
        assert_eq!(references.len(), NUM_OUTPUTS);
        assert!(statistics.iter().all(|s| s.from_cache));
    }

    // change root resource, one resource re-compiled.
    {
        change_resource(root_resource, project_dir);
        build.source_pull().expect("to pull changes");

        let CompileOutput {
            resources,
            references,
            statistics,
        } = build
            .compile_path(
                target.clone(),
                Target::Game,
                Platform::Windows,
                &Locale::new("en"),
            )
            .expect("successful compilation");

        assert_eq!(resources.len(), NUM_OUTPUTS);
        assert_eq!(references.len(), NUM_OUTPUTS);
        assert_eq!(statistics.iter().filter(|s| !s.from_cache).count(), 1);
    }

    // change resource E - which invalides 4 resources in total (E included).
    {
        let resource_e = resource_list[4];
        change_resource(resource_e, project_dir);
        build.source_pull().expect("to pull changes");

        let CompileOutput {
            resources,
            references,
            statistics,
        } = build
            .compile_path(target, Target::Game, Platform::Windows, &Locale::new("en"))
            .expect("successful compilation");

        assert_eq!(resources.len(), 5);
        assert_eq!(references.len(), 5);
        assert_eq!(statistics.iter().filter(|s| !s.from_cache).count(), 4);
    }
}

#[test]
fn link() {
    let work_dir = tempfile::tempdir().unwrap();
    let project_dir = work_dir.path();
    let mut resources = setup_registry();

    let parent_id = {
        let mut project = Project::create_new(project_dir).expect("new project");

        let child_handle = resources
            .new_resource(refs_resource::TYPE_ID)
            .expect("valid resource");
        let child = child_handle
            .get_mut::<refs_resource::TestResource>(&mut resources)
            .expect("existing resource");
        child.content = String::from("test child content");
        let child_id = project
            .add_resource(
                ResourceName::from("child"),
                refs_resource::TYPE_ID,
                &child_handle,
                &mut resources,
            )
            .unwrap();

        let parent_handle = resources
            .new_resource(refs_resource::TYPE_ID)
            .expect("valid resource");
        let parent = parent_handle
            .get_mut::<refs_resource::TestResource>(&mut resources)
            .expect("existing resource");
        parent.content = String::from("test parent content");
        parent.build_deps = vec![ResourcePathId::from(child_id).transform(refs_resource::TYPE_ID)];
        project
            .add_resource(
                ResourceName::from("parent"),
                refs_resource::TYPE_ID,
                &parent_handle,
                &mut resources,
            )
            .unwrap()
    };

    let contentstore_path = ContentStoreAddr::from(work_dir.path());
    let mut build = DataBuildOptions::new(project_dir.join(TEST_BUILDINDEX_FILENAME))
        .content_store(&contentstore_path)
        .compiler_dir(target_dir())
        .create(project_dir)
        .expect("to create index");

    build.source_pull().unwrap();

    // for now each resource is a separate file so we need to validate that the compile output and link output produce the same number of resources

    let target = ResourcePathId::from(parent_id).transform(refs_resource::TYPE_ID);
    let compile_output = build
        .compile_path(target, Target::Game, Platform::Windows, &Locale::new("en"))
        .expect("successful compilation");

    assert_eq!(compile_output.resources.len(), 2);
    assert_eq!(compile_output.references.len(), 1);

    let link_output = build
        .link(&compile_output.resources, &compile_output.references)
        .expect("successful linking");

    assert_eq!(compile_output.resources.len(), link_output.len());

    // link output checksum must be different from compile output checksum...
    for obj in &compile_output.resources {
        assert!(!link_output
            .iter()
            .any(|compiled| compiled.checksum == obj.compiled_checksum));
    }

    // ... and each output resource need to exist as exactly one resource object (although having different checksum).
    for output in link_output {
        assert_eq!(
            compile_output
                .resources
                .iter()
                .filter(|obj| obj.compiled_path == output.path)
                .count(),
            1
        );
    }
}

#[test]
fn verify_manifest() {
    let work_dir = tempfile::tempdir().unwrap();
    let project_dir = work_dir.path();
    let mut resources = setup_registry();

    // child_id <- test(child_id) <- parent_id = test(parent_id)
    let parent_resource = {
        let mut project = Project::create_new(project_dir).expect("new project");
        let child_id = project
            .add_resource(
                ResourceName::from("child"),
                refs_resource::TYPE_ID,
                &resources.new_resource(refs_resource::TYPE_ID).unwrap(),
                &mut resources,
            )
            .unwrap();

        let child_handle = resources.new_resource(refs_resource::TYPE_ID).unwrap();
        child_handle
            .get_mut::<refs_resource::TestResource>(&mut resources)
            .unwrap()
            .build_deps
            .push(ResourcePathId::from(child_id).transform(refs_resource::TYPE_ID));

        project
            .add_resource(
                ResourceName::from("parent"),
                refs_resource::TYPE_ID,
                &child_handle,
                &mut resources,
            )
            .unwrap()
    };

    let contentstore_path = ContentStoreAddr::from(work_dir.path());
    let mut build = DataBuildOptions::new(project_dir.join(TEST_BUILDINDEX_FILENAME))
        .content_store(&contentstore_path)
        .compiler_dir(target_dir())
        .create(project_dir)
        .expect("to create index");

    build.source_pull().unwrap();

    let output_manifest_file = work_dir.path().join(&DataBuild::default_output_file());

    let derived = ResourcePathId::from(parent_resource).transform(refs_resource::TYPE_ID);
    let manifest = build
        .compile(
            derived,
            &output_manifest_file,
            Target::Game,
            Platform::Windows,
            &Locale::new("en"),
        )
        .unwrap();

    // both test(child_id) and test(parent_id) are separate resources.
    assert_eq!(manifest.compiled_resources.len(), 2);

    let content_store = HddContentStore::open(contentstore_path).expect("valid content store");
    for checksum in manifest.compiled_resources.iter().map(|a| a.checksum) {
        assert!(content_store.exists(checksum));
    }

    assert!(output_manifest_file.exists());
    let read_manifest: Manifest = {
        let manifest_file = File::open(&output_manifest_file).unwrap();
        serde_json::from_reader(&manifest_file).unwrap()
    };

    assert_eq!(
        read_manifest.compiled_resources.len(),
        manifest.compiled_resources.len()
    );

    for resource in read_manifest.compiled_resources {
        assert!(manifest
            .compiled_resources
            .iter()
            .any(|res| res.checksum == resource.checksum));
    }
}
