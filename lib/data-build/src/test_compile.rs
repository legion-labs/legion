use std::fs::File;
use std::path::{Path, PathBuf};
use std::{env, vec};

use crate::databuild::CompileOutput;
use crate::{databuild::DataBuild, DataBuildOptions};
use integer_asset::{IntegerAsset, IntegerAssetLoader};
use legion_content_store::{ContentStore, ContentStoreAddr, HddContentStore};
use legion_data_compiler::{Locale, Manifest, Platform, Target};
use legion_data_offline::resource::ResourceRegistryOptions;
use legion_data_offline::{
    asset::AssetPathId,
    resource::{Project, ResourceId, ResourcePathName, ResourceProcessor, ResourceRegistry},
};
use legion_data_runtime::{AssetDescriptor, AssetLoader};
use multitext_resource::MultiTextResource;
use tempfile::TempDir;
use text_resource::{TextResource, TextResourceProc};

pub const TEST_BUILDINDEX_FILENAME: &str = "build.index";

fn setup_registry() -> ResourceRegistry {
    ResourceRegistryOptions::new()
        .add_type(
            refs_resource::TYPE_ID,
            Box::new(refs_resource::TestResourceProc {}),
        )
        .add_type(
            text_resource::TYPE_ID,
            Box::new(text_resource::TextResourceProc {}),
        )
        .add_type(
            multitext_resource::TYPE_ID,
            Box::new(multitext_resource::MultiTextResourceProc {}),
        )
        .create_registry()
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
    name: ResourcePathName,
    deps: &[AssetPathId],
    project: &mut Project,
    resources: &mut ResourceRegistry,
) -> ResourceId {
    let resource_b = {
        let res = resources
            .new_resource(refs_resource::TYPE_ID)
            .unwrap()
            .typed::<refs_resource::TestResource>();
        let resource = res.get_mut(resources).unwrap();
        resource.content = name.to_string(); // each resource needs unique content to generate a unique resource.
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
        .expect("to load resource")
        .typed::<refs_resource::TestResource>();

    let resource = handle.get_mut(&mut resources).expect("resource instance");
    resource.content.push_str(" more content");
    project
        .save_resource(resource_id, &handle, &mut resources)
        .expect("successful save");
}

fn setup_dir(work_dir: &TempDir) -> (PathBuf, PathBuf) {
    let project_dir = work_dir.path();
    let output_dir = project_dir.join("temp");
    std::fs::create_dir(&output_dir).unwrap();
    (project_dir.to_owned(), output_dir)
}

#[test]
fn compile_change_no_deps() {
    let work_dir = tempfile::tempdir().unwrap();
    let (project_dir, output_dir) = setup_dir(&work_dir);
    let mut resources = setup_registry();

    let (resource_id, resource_handle) = {
        let mut project = Project::create_new(&project_dir).expect("failed to create a project");

        let resource_handle = resources
            .new_resource(refs_resource::TYPE_ID)
            .unwrap()
            .typed::<refs_resource::TestResource>();
        let resource_id = project
            .add_resource(
                ResourcePathName::new("resource"),
                refs_resource::TYPE_ID,
                &resource_handle,
                &mut resources,
            )
            .unwrap();
        (resource_id, resource_handle)
    };

    let contentstore_path = ContentStoreAddr::from(output_dir.as_path());
    let mut config = DataBuildOptions::new(output_dir.join(TEST_BUILDINDEX_FILENAME));
    config
        .content_store(&contentstore_path)
        .compiler_dir(target_dir());

    let source = AssetPathId::from(resource_id);
    let target = source.push(refs_asset::RefsAsset::TYPE);

    // compile the resource..
    let original_checksum = {
        let mut build = config.create(&project_dir).expect("to create index");
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
        assert_eq!(compile_output.resources[0].compile_path, target);
        assert_eq!(
            compile_output.resources[0].compile_path,
            compile_output.resources[0].compiled_path
        );

        let original_checksum = compile_output.resources[0].compiled_checksum.get();

        let content_store =
            HddContentStore::open(contentstore_path.clone()).expect("valid content store");
        assert!(content_store.exists(original_checksum));

        original_checksum
    };

    // ..change resource..
    {
        let mut project = Project::open(project_dir).expect("failed to open project");

        resource_handle.get_mut(&mut resources).unwrap().content = String::from("new content");

        project
            .save_resource(resource_id, &resource_handle, &mut resources)
            .unwrap();
    }

    // ..re-compile changed resource..
    let modified_checksum = {
        let mut build = config.open().expect("to open index");
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
        assert_eq!(compile_output.resources[0].compile_path, target);
        assert_eq!(
            compile_output.resources[0].compile_path,
            compile_output.resources[0].compiled_path
        );

        let modified_checksum = compile_output.resources[0].compiled_checksum.get();

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

    let res_c = create_resource(
        ResourcePathName::new("C"),
        &[],
        &mut project,
        &mut resources,
    );
    let res_e = create_resource(
        ResourcePathName::new("E"),
        &[],
        &mut project,
        &mut resources,
    );
    let res_d = create_resource(
        ResourcePathName::new("D"),
        &[AssetPathId::from(res_e).push(refs_asset::RefsAsset::TYPE)],
        &mut project,
        &mut resources,
    );
    let res_b = create_resource(
        ResourcePathName::new("B"),
        &[
            AssetPathId::from(res_c).push(refs_asset::RefsAsset::TYPE),
            AssetPathId::from(res_e).push(refs_asset::RefsAsset::TYPE),
        ],
        &mut project,
        &mut resources,
    );
    let res_a = create_resource(
        ResourcePathName::new("A"),
        &[
            AssetPathId::from(res_b).push(refs_asset::RefsAsset::TYPE),
            AssetPathId::from(res_d).push(refs_asset::RefsAsset::TYPE),
        ],
        &mut project,
        &mut resources,
    );
    [res_a, res_b, res_c, res_d, res_e]
}

#[test]
fn intermediate_resource() {
    let work_dir = tempfile::tempdir().unwrap();
    let (project_dir, output_dir) = setup_dir(&work_dir);
    let mut resources = setup_registry();

    let source_magic_value = String::from("47");

    let source_id = {
        let mut project = Project::create_new(&project_dir).expect("failed to create a project");

        let resource_handle = resources
            .new_resource(text_resource::TYPE_ID)
            .unwrap()
            .typed::<TextResource>();
        resource_handle.get_mut(&mut resources).unwrap().content = source_magic_value.clone();
        project
            .add_resource(
                ResourcePathName::new("resource"),
                text_resource::TYPE_ID,
                &resource_handle,
                &mut resources,
            )
            .unwrap()
    };

    let cas_addr = ContentStoreAddr::from(output_dir.as_path());

    let mut build = DataBuildOptions::new(output_dir.join(TEST_BUILDINDEX_FILENAME))
        .content_store(&cas_addr)
        .compiler_dir(target_dir())
        .create(project_dir)
        .expect("new build index");

    let pulled = build.source_pull().expect("successful pull");
    assert_eq!(pulled, 1);

    let source_path = AssetPathId::from(source_id);
    let reversed_path = source_path.push(text_resource::TYPE_ID);
    let integer_path = reversed_path.push(integer_asset::IntegerAsset::TYPE);

    let compile_output = build
        .compile_path(
            integer_path.clone(),
            Target::Game,
            Platform::Windows,
            &Locale::new("en"),
        )
        .unwrap();

    assert_eq!(compile_output.resources.len(), 2); // intermediate and final result
    assert_eq!(compile_output.resources[0].compile_path, reversed_path);
    assert_eq!(compile_output.resources[1].compile_path, integer_path);
    assert!(compile_output
        .resources
        .iter()
        .all(|compiled| compiled.compile_path == compiled.compiled_path));

    let content_store = HddContentStore::open(cas_addr).expect("valid cas");

    // validate reversed
    {
        let checksum = compile_output.resources[0].compiled_checksum.get();
        assert!(content_store.exists(checksum));
        let resource_content = content_store.read(checksum).expect("asset content");

        let mut creator = TextResourceProc {};
        let resource = creator
            .read_resource(&mut &resource_content[..])
            .expect("loaded assets");
        let resource = resource.downcast_ref::<TextResource>().unwrap();

        assert_eq!(
            source_magic_value.chars().rev().collect::<String>(),
            resource.content
        );
    }

    // validate integer
    {
        let checksum: i128 = compile_output.resources[1].compiled_checksum.get();
        assert!(content_store.exists(checksum));
        let resource_content = content_store.read(checksum).expect("asset content");

        let mut loader = IntegerAssetLoader {};
        let resource = loader
            .load(
                integer_asset::IntegerAsset::TYPE,
                &mut &resource_content[..],
            )
            .expect("loaded assets");
        let resource = resource.downcast_ref::<IntegerAsset>().unwrap();

        let stringified = resource.magic_value.to_string();
        assert_eq!(
            source_magic_value.chars().rev().collect::<String>(),
            stringified
        );
    }
}

#[test]
fn unnamed_cache_use() {
    let work_dir = tempfile::tempdir().unwrap();
    let (project_dir, output_dir) = setup_dir(&work_dir);

    let resource_list = setup_project(&project_dir);
    let root_resource = resource_list[0];

    let mut build = DataBuildOptions::new(output_dir.join(TEST_BUILDINDEX_FILENAME))
        .content_store(&ContentStoreAddr::from(output_dir))
        .compiler_dir(target_dir())
        .create(&project_dir)
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
    const NUM_OUTPUTS: usize = 5;
    let target = AssetPathId::from(root_resource).push(refs_asset::RefsAsset::TYPE);

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
        change_resource(root_resource, &project_dir);
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
        change_resource(resource_e, &project_dir);
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
fn named_path_cache_use() {
    let work_dir = tempfile::tempdir().unwrap();
    let (project_dir, output_dir) = setup_dir(&work_dir);
    let mut resources = setup_registry();

    let magic_list = vec![String::from("47"), String::from("198")];

    let source_id = {
        let mut project = Project::create_new(&project_dir).expect("failed to create a project");

        let resource_handle = resources
            .new_resource(multitext_resource::TYPE_ID)
            .unwrap()
            .typed::<MultiTextResource>();
        resource_handle.get_mut(&mut resources).unwrap().text_list = magic_list.clone();
        project
            .add_resource(
                ResourcePathName::new("resource"),
                multitext_resource::TYPE_ID,
                &resource_handle,
                &mut resources,
            )
            .unwrap()
    };

    let cas_addr = ContentStoreAddr::from(output_dir.as_path());

    let mut build = DataBuildOptions::new(output_dir.join(TEST_BUILDINDEX_FILENAME))
        .content_store(&cas_addr)
        .compiler_dir(target_dir())
        .create(&project_dir)
        .expect("new build index");

    let pulled = build.source_pull().expect("successful pull");
    assert_eq!(pulled, 1);

    let source_path = AssetPathId::from(source_id);
    let split_text0_path = source_path.push_named(text_resource::TYPE_ID, "text_0");
    let split_text1_path = source_path.push_named(text_resource::TYPE_ID, "text_1");
    let integer_path_0 = split_text0_path.push(integer_asset::IntegerAsset::TYPE);
    let integer_path_1 = split_text1_path.push(integer_asset::IntegerAsset::TYPE);

    //
    // multitext_resource -> text_resource("text_0") -> integer_asset <= "integer path 0"
    //                    -> text_resource("text_1") -> integer_asset <= "integer path 1"
    //

    // compile "integer path 0"
    let compile_output = build
        .compile_path(
            integer_path_0.clone(),
            Target::Game,
            Platform::Windows,
            &Locale::new("en"),
        )
        .unwrap();

    assert_eq!(compile_output.resources.len(), magic_list.len() + 1);
    assert!(compile_output.statistics.iter().all(|s| !s.from_cache));
    assert!(compile_output
        .resources
        .iter()
        .all(|r| !r.compile_path.is_named()));

    let compiled_text0 = compile_output
        .resources
        .iter()
        .find(|&info| info.compiled_path == split_text0_path)
        .unwrap();

    assert_eq!(compiled_text0.compile_path, split_text0_path.to_unnamed());

    let compiled_integer = compile_output
        .resources
        .iter()
        .find(|&info| info.compiled_path == integer_path_0)
        .unwrap();

    assert_eq!(compiled_integer.compile_path, integer_path_0);
    assert_eq!(compiled_integer.compiled_path, integer_path_0);

    let content_store = HddContentStore::open(cas_addr).expect("valid cas");

    // validate integer
    {
        let checksum = compiled_integer.compiled_checksum.get();
        assert!(content_store.exists(checksum));
        let resource_content = content_store.read(checksum).expect("asset content");

        let mut loader = IntegerAssetLoader {};
        let resource = loader
            .load(
                integer_asset::IntegerAsset::TYPE,
                &mut &resource_content[..],
            )
            .expect("loaded assets");
        let resource = resource.downcast_ref::<IntegerAsset>().unwrap();

        let stringified = resource.magic_value.to_string();
        assert_eq!(magic_list[0], stringified);
    }

    // compile "integer path 1"
    let compile_output = build
        .compile_path(
            integer_path_1.clone(),
            Target::Game,
            Platform::Windows,
            &Locale::new("en"),
        )
        .unwrap();

    assert_eq!(compile_output.resources.len(), magic_list.len() + 1);
    assert_eq!(
        compile_output
            .statistics
            .iter()
            .filter(|s| s.from_cache)
            .count(),
        2 // both "text_0" and "text_1"
    );
    assert!(compile_output
        .resources
        .iter()
        .all(|r| !r.compile_path.is_named()));

    // recompile "integer path 0" - all from cache
    let compile_output = build
        .compile_path(
            integer_path_0.clone(),
            Target::Game,
            Platform::Windows,
            &Locale::new("en"),
        )
        .unwrap();

    assert_eq!(compile_output.resources.len(), magic_list.len() + 1);
    assert!(compile_output.statistics.iter().all(|s| s.from_cache));
    assert!(compile_output
        .resources
        .iter()
        .all(|r| !r.compile_path.is_named()));

    // change "text_1" of source resource multitext resource..
    {
        let mut project = Project::open(&project_dir).expect("failed to open project");
        let mut resources = setup_registry();

        let handle = project
            .load_resource(source_id, &mut resources)
            .expect("to load resource")
            .typed::<multitext_resource::MultiTextResource>();

        let resource = handle.get_mut(&mut resources).expect("resource instance");
        resource.text_list[1] = String::from("852");
        project
            .save_resource(source_id, &handle, &mut resources)
            .expect("successful save");

        let pulled = build.source_pull().expect("pulled change");
        assert_eq!(pulled, 1);
    }

    let compile_output = build
        .compile_path(
            integer_path_0.clone(),
            Target::Game,
            Platform::Windows,
            &Locale::new("en"),
        )
        .unwrap();

    // ..recompiled: multitext -> text_0, text_1
    // ..from cache: text_0 -> integer
    assert_eq!(compile_output.resources.len(), magic_list.len() + 1);
    assert_eq!(
        compile_output
            .statistics
            .iter()
            .filter(|s| s.from_cache)
            .count(),
        1
    );
    assert!(compile_output
        .resources
        .iter()
        .all(|r| !r.compile_path.is_named()));

    // change "text_0" and "text_1" of source resource multitext resource..
    {
        let mut project = Project::open(project_dir).expect("failed to open project");
        let mut resources = setup_registry();

        let handle = project
            .load_resource(source_id, &mut resources)
            .expect("to load resource")
            .typed::<multitext_resource::MultiTextResource>();

        let resource = handle.get_mut(&mut resources).expect("resource instance");
        resource.text_list[0] = String::from("734");
        resource.text_list[1] = String::from("1");
        project
            .save_resource(source_id, &handle, &mut resources)
            .expect("successful save");

        let pulled = build.source_pull().expect("pulled change");
        assert_eq!(pulled, 1);
    }

    // compile from "text_0"
    let compile_output = build
        .compile_path(
            integer_path_0.clone(),
            Target::Game,
            Platform::Windows,
            &Locale::new("en"),
        )
        .unwrap();

    // ..recompiled: multitext -> text_0, text_1, text_0 -> integer
    // ..from cache: none
    assert_eq!(compile_output.resources.len(), magic_list.len() + 1);
    assert_eq!(
        compile_output
            .statistics
            .iter()
            .filter(|s| s.from_cache)
            .count(),
        0
    );
    assert!(compile_output
        .resources
        .iter()
        .all(|r| !r.compile_path.is_named()));

    // compile from "text_1"
    let compile_output = build
        .compile_path(
            integer_path_1,
            Target::Game,
            Platform::Windows,
            &Locale::new("en"),
        )
        .unwrap();

    // ..recompiled: text_1 -> integer
    // ..from cache: multitext -> text_0, text_1
    assert_eq!(compile_output.resources.len(), magic_list.len() + 1);
    assert_eq!(
        compile_output
            .statistics
            .iter()
            .filter(|s| s.from_cache)
            .count(),
        2
    );
    assert!(compile_output
        .resources
        .iter()
        .all(|r| !r.compile_path.is_named()));
}

#[test]
fn link() {
    let work_dir = tempfile::tempdir().unwrap();
    let (project_dir, output_dir) = setup_dir(&work_dir);
    let mut resources = setup_registry();

    let parent_id = {
        let mut project = Project::create_new(&project_dir).expect("new project");

        let child_handle = resources
            .new_resource(refs_resource::TYPE_ID)
            .expect("valid resource")
            .typed::<refs_resource::TestResource>();
        let child = child_handle
            .get_mut(&mut resources)
            .expect("existing resource");
        child.content = String::from("test child content");
        let child_id = project
            .add_resource(
                ResourcePathName::new("child"),
                refs_resource::TYPE_ID,
                &child_handle,
                &mut resources,
            )
            .unwrap();

        let parent_handle = resources
            .new_resource(refs_resource::TYPE_ID)
            .expect("valid resource")
            .typed::<refs_resource::TestResource>();
        let parent = parent_handle
            .get_mut(&mut resources)
            .expect("existing resource");
        parent.content = String::from("test parent content");
        parent.build_deps = vec![AssetPathId::from(child_id).push(refs_asset::RefsAsset::TYPE)];
        project
            .add_resource(
                ResourcePathName::new("parent"),
                refs_resource::TYPE_ID,
                &parent_handle,
                &mut resources,
            )
            .unwrap()
    };

    let contentstore_path = ContentStoreAddr::from(output_dir.as_path());
    let mut build = DataBuildOptions::new(output_dir.join(TEST_BUILDINDEX_FILENAME))
        .content_store(&contentstore_path)
        .compiler_dir(target_dir())
        .create(&project_dir)
        .expect("to create index");

    build.source_pull().unwrap();

    // for now each resource is a separate file so we need to validate that the compile output and link output produce the same number of resources

    let target = AssetPathId::from(parent_id).push(refs_asset::RefsAsset::TYPE);
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
    let (project_dir, output_dir) = setup_dir(&work_dir);
    let mut resources = setup_registry();

    // child_id <- test(child_id) <- parent_id = test(parent_id)
    let parent_resource = {
        let mut project = Project::create_new(&project_dir).expect("new project");
        let child_id = project
            .add_resource(
                ResourcePathName::new("child"),
                refs_resource::TYPE_ID,
                &resources.new_resource(refs_resource::TYPE_ID).unwrap(),
                &mut resources,
            )
            .unwrap();

        let child_handle = resources
            .new_resource(refs_resource::TYPE_ID)
            .unwrap()
            .typed::<refs_resource::TestResource>();
        child_handle
            .get_mut(&mut resources)
            .unwrap()
            .build_deps
            .push(AssetPathId::from(child_id).push(refs_asset::RefsAsset::TYPE));

        project
            .add_resource(
                ResourcePathName::new("parent"),
                refs_resource::TYPE_ID,
                &child_handle,
                &mut resources,
            )
            .unwrap()
    };

    let contentstore_path = ContentStoreAddr::from(output_dir.as_path());
    let mut build = DataBuildOptions::new(output_dir.join(TEST_BUILDINDEX_FILENAME))
        .content_store(&contentstore_path)
        .compiler_dir(target_dir())
        .create(project_dir)
        .expect("to create index");

    build.source_pull().unwrap();

    let output_manifest_file = work_dir.path().join(&DataBuild::default_output_file());

    let compile_path = AssetPathId::from(parent_resource).push(refs_asset::RefsAsset::TYPE);
    let manifest = build
        .compile(
            compile_path,
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
        assert!(content_store.exists(checksum.get()));
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
