#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::{env, vec};

    use integer_asset::{IntegerAsset, IntegerAssetLoader};
    use lgn_content_store::{ContentStore, ContentStoreAddr, HddContentStore};
    use lgn_content_store2::{ContentProvider, MemoryProvider};
    use lgn_data_compiler::compiler_api::CompilationEnv;
    use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
    use lgn_data_compiler::{Locale, Platform, Target};
    use lgn_data_offline::resource::ResourceRegistryOptions;
    use lgn_data_offline::{
        resource::{Project, ResourcePathName, ResourceProcessor, ResourceRegistry},
        ResourcePathId,
    };
    use lgn_data_runtime::{AssetLoader, Resource, ResourceTypeAndId};
    use lgn_source_control::{LocalRepositoryIndex, RepositoryIndex};
    use multitext_resource::MultiTextResource;
    use tempfile::TempDir;
    use text_resource::{TextResource, TextResourceProc};

    use crate::databuild::CompileOutput;
    use crate::DataBuildOptions;

    pub(crate) async fn setup_dir(
        work_dir: &TempDir,
    ) -> (
        PathBuf,
        PathBuf,
        LocalRepositoryIndex,
        Arc<Box<dyn ContentProvider + Send + Sync>>,
    ) {
        let project_dir = work_dir.path();
        let output_dir = project_dir.join("temp");
        std::fs::create_dir_all(&output_dir).unwrap();

        let repository_index = LocalRepositoryIndex::new(project_dir.join("remote"))
            .await
            .unwrap();
        let content_provider: Arc<Box<dyn ContentProvider + Send + Sync>> =
            Arc::new(Box::new(MemoryProvider::new()));

        (
            project_dir.to_owned(),
            output_dir,
            repository_index,
            content_provider,
        )
    }

    fn setup_registry() -> Arc<tokio::sync::Mutex<ResourceRegistry>> {
        ResourceRegistryOptions::new()
            .add_type::<refs_resource::TestResource>()
            .add_type::<text_resource::TextResource>()
            .add_type::<multitext_resource::MultiTextResource>()
            .create_async_registry()
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

    async fn create_resource(
        name: ResourcePathName,
        deps: &[ResourcePathId],
        project: &mut Project,
        resources: &mut ResourceRegistry,
    ) -> ResourceTypeAndId {
        let resource_b = {
            let res = resources
                .new_resource(refs_resource::TestResource::TYPE)
                .unwrap()
                .typed::<refs_resource::TestResource>();
            let resource = res.get_mut(resources).unwrap();
            resource.content = name.to_string(); // each resource needs unique content to generate a unique resource.
            resource.build_deps.extend_from_slice(deps);
            res
        };
        project
            .add_resource(
                name,
                refs_resource::TestResource::TYPENAME,
                refs_resource::TestResource::TYPE,
                &resource_b,
                resources,
            )
            .await
            .unwrap()
    }

    async fn change_resource(
        resource_id: ResourceTypeAndId,
        project_dir: &Path,
        repository_index: impl RepositoryIndex,
        content_provider: Arc<Box<dyn ContentProvider + Send + Sync>>,
    ) {
        let mut project = Project::open(project_dir, repository_index, content_provider)
            .await
            .expect("failed to open project");
        let resources = setup_registry();
        let mut resources = resources.lock().await;

        let handle = project
            .load_resource(resource_id, &mut resources)
            .expect("to load resource")
            .typed::<refs_resource::TestResource>();

        let resource = handle.get_mut(&mut resources).expect("resource instance");
        resource.content.push_str(" more content");
        project
            .save_resource(resource_id, &handle, &mut resources)
            .await
            .expect("successful save");
    }

    fn test_env() -> CompilationEnv {
        CompilationEnv {
            target: Target::Game,
            platform: Platform::Windows,
            locale: Locale::new("en"),
        }
    }

    #[tokio::test]
    async fn compile_change_no_deps() {
        let work_dir = tempfile::tempdir().unwrap();
        let (project_dir, output_dir, repository_index, content_provider) =
            setup_dir(&work_dir).await;
        let resources = setup_registry();
        let mut resources = resources.lock().await;

        let (resource_id, resource_handle) = {
            let mut project =
                Project::create_with_remote_mock(&project_dir, Arc::clone(&content_provider))
                    .await
                    .expect("failed to create a project");

            let resource_handle = resources
                .new_resource(refs_resource::TestResource::TYPE)
                .unwrap()
                .typed::<refs_resource::TestResource>();
            let resource_id = project
                .add_resource(
                    ResourcePathName::new("resource"),
                    refs_resource::TestResource::TYPENAME,
                    refs_resource::TestResource::TYPE,
                    &resource_handle,
                    &mut resources,
                )
                .await
                .unwrap();
            (resource_id, resource_handle)
        };

        let contentstore_path = ContentStoreAddr::from(output_dir.as_path());
        let config = DataBuildOptions::new_with_sqlite_output(
            &output_dir,
            CompilerRegistryOptions::local_compilers(target_dir()),
            Arc::clone(&data_content_provider),
        );

        let source = ResourcePathId::from(resource_id);
        let target = source.push(refs_asset::RefsAsset::TYPE);

        // compile the resource..
        let original_checksum = {
            let (mut build, project) = config
                .create_with_project(
                    &project_dir,
                    &repository_index,
                    Arc::clone(&content_provider),
                )
                .await
                .expect("to create index");
            build
                .source_pull(&project)
                .await
                .expect("failed to pull from project");

            let compile_output = build
                .compile_path(target.clone(), &test_env(), None)
                .await
                .unwrap();

            assert_eq!(compile_output.resources.len(), 1);
            assert_eq!(compile_output.references.len(), 0);
            assert_eq!(compile_output.resources[0].compile_path, target);
            assert_eq!(
                compile_output.resources[0].compile_path,
                compile_output.resources[0].compiled_path
            );

            let original_checksum = compile_output.resources[0].compiled_checksum;

            let content_store =
                HddContentStore::open(contentstore_path.clone()).expect("valid content store");
            assert!(content_store.exists(original_checksum).await);

            original_checksum
        };

        // ..change resource..
        {
            let mut project = Project::open(
                &project_dir,
                &repository_index,
                Arc::clone(&content_provider),
            )
            .await
            .expect("failed to open project");

            resource_handle.get_mut(&mut resources).unwrap().content = String::from("new content");

            project
                .save_resource(resource_id, &resource_handle, &mut resources)
                .await
                .unwrap();
        }

        // ..re-compile changed resource..
        let modified_checksum = {
            let config = DataBuildOptions::new_with_sqlite_output(
                output_dir,
                CompilerRegistryOptions::local_compilers(target_dir()),
            )
            .content_store(&contentstore_path);

            let project = Project::open(
                project_dir,
                &repository_index,
                Arc::clone(&content_provider),
            )
            .await
            .expect("failed to open project");

            let mut build = config.open(&project).await.expect("to open index");
            build
                .source_pull(&project)
                .await
                .expect("failed to pull from project");
            let compile_output = build
                .compile_path(target.clone(), &test_env(), None)
                .await
                .unwrap();

            assert_eq!(compile_output.resources.len(), 1);
            assert_eq!(compile_output.resources[0].compile_path, target);
            assert_eq!(
                compile_output.resources[0].compile_path,
                compile_output.resources[0].compiled_path
            );

            let modified_checksum = compile_output.resources[0].compiled_checksum;

            let content_store =
                HddContentStore::open(contentstore_path).expect("valid content store");
            assert!(content_store.exists(original_checksum).await);
            assert!(content_store.exists(modified_checksum).await);

            modified_checksum
        };

        assert_ne!(original_checksum, modified_checksum);
    }

    /// Creates a project with 5 resources with dependencies setup as depicted
    /// below. t(A) depicts a dependency on a `derived resource A` transformed  by
    /// `t`. Returns an array of resources from A to E where A is at index 0.
    //
    // t(A) -> A -> t(B) -> B -> t(C) -> C
    //         |            |
    //         V            |
    //       t(D)           |
    //         |            |
    //         V            V
    //         D -------> t(E) -> E
    //
    async fn setup_project(
        project_dir: impl AsRef<Path>,
        content_provider: Arc<Box<dyn ContentProvider + Send + Sync>>,
    ) -> [ResourceTypeAndId; 5] {
        let mut project = Project::create_with_remote_mock(project_dir.as_ref(), content_provider)
            .await
            .expect("failed to create a project");

        let resources = setup_registry();
        let mut resources = resources.lock().await;

        let res_c = create_resource(
            ResourcePathName::new("C"),
            &[],
            &mut project,
            &mut resources,
        )
        .await;
        let res_e = create_resource(
            ResourcePathName::new("E"),
            &[],
            &mut project,
            &mut resources,
        )
        .await;
        let res_d = create_resource(
            ResourcePathName::new("D"),
            &[ResourcePathId::from(res_e).push(refs_asset::RefsAsset::TYPE)],
            &mut project,
            &mut resources,
        )
        .await;
        let res_b = create_resource(
            ResourcePathName::new("B"),
            &[
                ResourcePathId::from(res_c).push(refs_asset::RefsAsset::TYPE),
                ResourcePathId::from(res_e).push(refs_asset::RefsAsset::TYPE),
            ],
            &mut project,
            &mut resources,
        )
        .await;
        let res_a = create_resource(
            ResourcePathName::new("A"),
            &[
                ResourcePathId::from(res_b).push(refs_asset::RefsAsset::TYPE),
                ResourcePathId::from(res_d).push(refs_asset::RefsAsset::TYPE),
            ],
            &mut project,
            &mut resources,
        )
        .await;
        [res_a, res_b, res_c, res_d, res_e]
    }

    #[tokio::test]
    async fn intermediate_resource() {
        let work_dir = tempfile::tempdir().unwrap();
        let (project_dir, output_dir, repository_index, content_provider) =
            setup_dir(&work_dir).await;
        let resources = setup_registry();
        let mut resources = resources.lock().await;

        let source_magic_value = String::from("47");

        let source_id = {
            let mut project =
                Project::create_with_remote_mock(&project_dir, Arc::clone(&content_provider))
                    .await
                    .expect("failed to create a project");

            let resource_handle = resources
                .new_resource(text_resource::TextResource::TYPE)
                .unwrap()
                .typed::<TextResource>();
            resource_handle.get_mut(&mut resources).unwrap().content = source_magic_value.clone();
            project
                .add_resource(
                    ResourcePathName::new("resource"),
                    text_resource::TextResource::TYPENAME,
                    text_resource::TextResource::TYPE,
                    &resource_handle,
                    &mut resources,
                )
                .await
                .unwrap()
        };

        let cas_addr = ContentStoreAddr::from(output_dir.as_path());

        let (mut build, project) = DataBuildOptions::new_with_sqlite_output(
            output_dir,
            CompilerRegistryOptions::local_compilers(target_dir()),
        )
        .content_store(&cas_addr)
        .create_with_project(project_dir, &repository_index, content_provider)
        .await
        .expect("new build index");

        build.source_pull(&project).await.expect("successful pull");

        let source_path = ResourcePathId::from(source_id);
        let reversed_path = source_path.push(text_resource::TextResource::TYPE);
        let integer_path = reversed_path.push(integer_asset::IntegerAsset::TYPE);

        let compile_output = build
            .compile_path(integer_path.clone(), &test_env(), None)
            .await
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
            let checksum = compile_output.resources[0].compiled_checksum;
            assert!(content_store.exists(checksum).await);
            let resource_content = content_store
                .read(checksum)
                .await
                .expect("resource content");

            let mut creator = TextResourceProc {};
            let resource = creator
                .read_resource(&mut &resource_content[..])
                .expect("loaded resource");
            let resource = resource.downcast_ref::<TextResource>().unwrap();

            assert_eq!(
                source_magic_value.chars().rev().collect::<String>(),
                resource.content
            );
        }

        // validate integer
        {
            let checksum = compile_output.resources[1].compiled_checksum;
            assert!(content_store.exists(checksum).await);
            let resource_content = content_store.read(checksum).await.expect("asset content");

            let mut loader = IntegerAssetLoader {};
            let resource = loader
                .load(&mut &resource_content[..])
                .expect("loaded resource");
            let resource = resource.downcast_ref::<IntegerAsset>().unwrap();

            let stringified = resource.magic_value.to_string();
            assert_eq!(
                source_magic_value.chars().rev().collect::<String>(),
                stringified
            );
        }
    }

    #[tokio::test]
    async fn unnamed_cache_use() {
        let work_dir = tempfile::tempdir().unwrap();
        let (project_dir, output_dir, repository_index, content_provider) =
            setup_dir(&work_dir).await;

        let resource_list = setup_project(&project_dir, Arc::clone(&content_provider)).await;
        let root_resource = resource_list[0];

        let (mut build, project) = DataBuildOptions::new_with_sqlite_output(
            &output_dir,
            CompilerRegistryOptions::local_compilers(target_dir()),
        )
        .content_store(&ContentStoreAddr::from(output_dir))
        .create_with_project(
            &project_dir,
            &repository_index,
            Arc::clone(&content_provider),
        )
        .await
        .expect("new build index");
        build.source_pull(&project).await.expect("successful pull");

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
        let target = ResourcePathId::from(root_resource).push(refs_asset::RefsAsset::TYPE);

        // first run - none of the resources from cache.
        {
            let CompileOutput {
                resources,
                references,
                statistics,
            } = build
                .compile_path(target.clone(), &test_env(), None)
                .await
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
                .compile_path(target.clone(), &test_env(), None)
                .await
                .expect("successful compilation");

            assert_eq!(resources.len(), NUM_OUTPUTS);
            assert_eq!(references.len(), NUM_OUTPUTS);
            assert!(statistics.iter().all(|s| s.from_cache));
        }

        // change root resource, one resource re-compiled.
        {
            change_resource(
                root_resource,
                &project_dir,
                &repository_index,
                Arc::clone(&content_provider),
            )
            .await;
            build.source_pull(&project).await.expect("to pull changes");

            let CompileOutput {
                resources,
                references,
                statistics,
            } = build
                .compile_path(target.clone(), &test_env(), None)
                .await
                .expect("successful compilation");

            assert_eq!(resources.len(), NUM_OUTPUTS);
            assert_eq!(references.len(), NUM_OUTPUTS);
            assert_eq!(statistics.iter().filter(|s| !s.from_cache).count(), 1);
        }

        // change resource E - which invalides 4 resources in total (E included).
        {
            let resource_e = resource_list[4];
            change_resource(
                resource_e,
                &project_dir,
                &repository_index,
                content_provider,
            )
            .await;
            build.source_pull(&project).await.expect("to pull changes");

            let CompileOutput {
                resources,
                references,
                statistics,
            } = build
                .compile_path(target, &test_env(), None)
                .await
                .expect("successful compilation");

            assert_eq!(resources.len(), 5);
            assert_eq!(references.len(), 5);
            assert_eq!(statistics.iter().filter(|s| !s.from_cache).count(), 4);
        }
    }

    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn named_path_cache_use() {
        let work_dir = tempfile::tempdir().unwrap();
        let (project_dir, output_dir, repository_index, content_provider) =
            setup_dir(&work_dir).await;
        let resources = setup_registry();
        let mut resources = resources.lock().await;

        let magic_list = vec![String::from("47"), String::from("198")];

        let source_id = {
            let mut project =
                Project::create_with_remote_mock(&project_dir, Arc::clone(&content_provider))
                    .await
                    .expect("failed to create a project");

            let resource_handle = resources
                .new_resource(multitext_resource::MultiTextResource::TYPE)
                .unwrap()
                .typed::<MultiTextResource>();
            resource_handle.get_mut(&mut resources).unwrap().text_list = magic_list.clone();
            project
                .add_resource(
                    ResourcePathName::new("resource"),
                    multitext_resource::MultiTextResource::TYPENAME,
                    multitext_resource::MultiTextResource::TYPE,
                    &resource_handle,
                    &mut resources,
                )
                .await
                .unwrap()
        };

        let cas_addr = ContentStoreAddr::from(output_dir.as_path());

        let (mut build, project) = DataBuildOptions::new_with_sqlite_output(
            output_dir,
            CompilerRegistryOptions::local_compilers(target_dir()),
        )
        .content_store(&cas_addr)
        .create_with_project(
            &project_dir,
            &repository_index,
            Arc::clone(&content_provider),
        )
        .await
        .expect("new build index");

        build.source_pull(&project).await.expect("successful pull");

        let source_path = ResourcePathId::from(source_id);
        let split_text0_path = source_path.push_named(text_resource::TextResource::TYPE, "text_0");
        let split_text1_path = source_path.push_named(text_resource::TextResource::TYPE, "text_1");
        let integer_path_0 = split_text0_path.push(integer_asset::IntegerAsset::TYPE);
        let integer_path_1 = split_text1_path.push(integer_asset::IntegerAsset::TYPE);

        //
        // multitext_resource -> text_resource("text_0") -> integer_asset <= "integer
        // path 0"                    -> text_resource("text_1") -> integer_asset <=
        // "integer path 1"
        //

        // compile "integer path 0"
        let compile_output = build
            .compile_path(integer_path_0.clone(), &test_env(), None)
            .await
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
            let checksum = compiled_integer.compiled_checksum;
            assert!(content_store.exists(checksum).await);
            let resource_content = content_store.read(checksum).await.expect("asset content");

            let mut loader = IntegerAssetLoader {};
            let resource = loader
                .load(&mut &resource_content[..])
                .expect("loaded resource");
            let resource = resource.downcast_ref::<IntegerAsset>().unwrap();

            let stringified = resource.magic_value.to_string();
            assert_eq!(magic_list[0], stringified);
        }

        // compile "integer path 1"
        let compile_output = build
            .compile_path(integer_path_1.clone(), &test_env(), None)
            .await
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
            .compile_path(integer_path_0.clone(), &test_env(), None)
            .await
            .unwrap();

        assert_eq!(compile_output.resources.len(), magic_list.len() + 1);
        assert!(compile_output.statistics.iter().all(|s| s.from_cache));
        assert!(compile_output
            .resources
            .iter()
            .all(|r| !r.compile_path.is_named()));

        // change "text_1" of source resource multitext resource..
        {
            let mut project = Project::open(
                &project_dir,
                &repository_index,
                Arc::clone(&content_provider),
            )
            .await
            .expect("failed to open project");
            let resources = setup_registry();
            let mut resources = resources.lock().await;

            let handle = project
                .load_resource(source_id, &mut resources)
                .expect("to load resource")
                .typed::<multitext_resource::MultiTextResource>();

            let resource = handle.get_mut(&mut resources).expect("resource instance");
            resource.text_list[1] = String::from("852");
            project
                .save_resource(source_id, &handle, &mut resources)
                .await
                .expect("successful save");

            build.source_pull(&project).await.expect("pulled change");
        }

        let compile_output = build
            .compile_path(integer_path_0.clone(), &test_env(), None)
            .await
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
            let mut project = Project::open(
                project_dir,
                &repository_index,
                Arc::clone(&content_provider),
            )
            .await
            .expect("failed to open project");
            let resources = setup_registry();
            let mut resources = resources.lock().await;

            let handle = project
                .load_resource(source_id, &mut resources)
                .expect("to load resource")
                .typed::<multitext_resource::MultiTextResource>();

            let resource = handle.get_mut(&mut resources).expect("resource instance");
            resource.text_list[0] = String::from("734");
            resource.text_list[1] = String::from("1");
            project
                .save_resource(source_id, &handle, &mut resources)
                .await
                .expect("successful save");

            build.source_pull(&project).await.expect("pulled change");
        }

        // compile from "text_0"
        let compile_output = build
            .compile_path(integer_path_0.clone(), &test_env(), None)
            .await
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
            .compile_path(integer_path_1, &test_env(), None)
            .await
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

    #[tokio::test]
    async fn link() {
        let work_dir = tempfile::tempdir().unwrap();
        let (project_dir, output_dir, repository_index, content_provider) =
            setup_dir(&work_dir).await;
        let resources = setup_registry();
        let mut resources = resources.lock().await;

        let parent_id = {
            let mut project =
                Project::create_with_remote_mock(&project_dir, Arc::clone(&content_provider))
                    .await
                    .expect("new project");

            let child_handle = resources
                .new_resource(refs_resource::TestResource::TYPE)
                .expect("valid resource")
                .typed::<refs_resource::TestResource>();
            let child = child_handle
                .get_mut(&mut resources)
                .expect("existing resource");
            child.content = String::from("test child content");
            let child_id = project
                .add_resource(
                    ResourcePathName::new("child"),
                    refs_resource::TestResource::TYPENAME,
                    refs_resource::TestResource::TYPE,
                    &child_handle,
                    &mut resources,
                )
                .await
                .unwrap();

            let parent_handle = resources
                .new_resource(refs_resource::TestResource::TYPE)
                .expect("valid resource")
                .typed::<refs_resource::TestResource>();
            let parent = parent_handle
                .get_mut(&mut resources)
                .expect("existing resource");
            parent.content = String::from("test parent content");
            parent.build_deps =
                vec![ResourcePathId::from(child_id).push(refs_asset::RefsAsset::TYPE)];
            project
                .add_resource(
                    ResourcePathName::new("parent"),
                    refs_resource::TestResource::TYPENAME,
                    refs_resource::TestResource::TYPE,
                    &parent_handle,
                    &mut resources,
                )
                .await
                .unwrap()
        };

        let contentstore_path = ContentStoreAddr::from(output_dir.as_path());
        let (mut build, project) = DataBuildOptions::new_with_sqlite_output(
            output_dir,
            CompilerRegistryOptions::local_compilers(target_dir()),
        )
        .content_store(&contentstore_path)
        .create_with_project(&project_dir, &repository_index, content_provider)
        .await
        .expect("to create index");

        build.source_pull(&project).await.unwrap();

        // for now each resource is a separate file so we need to validate that the
        // compile output and link output produce the same number of resources

        let target = ResourcePathId::from(parent_id).push(refs_asset::RefsAsset::TYPE);
        let compile_output = build
            .compile_path(target, &test_env(), None)
            .await
            .expect("successful compilation");

        assert_eq!(compile_output.resources.len(), 2);
        assert_eq!(compile_output.references.len(), 1);

        let link_output = build
            .link(&compile_output.resources, &compile_output.references)
            .await
            .expect("successful linking");

        assert_eq!(compile_output.resources.len(), link_output.len());

        // link output checksum must be different from compile output checksum...
        for obj in &compile_output.resources {
            assert!(!link_output
                .iter()
                .any(|compiled| compiled.checksum == obj.compiled_checksum));
        }

        // ... and each output resource need to exist as exactly one resource object
        // (although having different checksum).
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

    #[tokio::test]
    async fn verify_manifest() {
        let work_dir = tempfile::tempdir().unwrap();
        let (project_dir, output_dir, repository_index, content_provider) =
            setup_dir(&work_dir).await;
        let resources = setup_registry();
        let mut resources = resources.lock().await;

        // child_id <- test(child_id) <- parent_id = test(parent_id)
        let parent_resource = {
            let mut project =
                Project::create_with_remote_mock(&project_dir, Arc::clone(&content_provider))
                    .await
                    .expect("new project");
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

            let child_handle = resources
                .new_resource(refs_resource::TestResource::TYPE)
                .unwrap()
                .typed::<refs_resource::TestResource>();
            child_handle
                .get_mut(&mut resources)
                .unwrap()
                .build_deps
                .push(ResourcePathId::from(child_id).push(refs_asset::RefsAsset::TYPE));

            project
                .add_resource(
                    ResourcePathName::new("parent"),
                    refs_resource::TestResource::TYPENAME,
                    refs_resource::TestResource::TYPE,
                    &child_handle,
                    &mut resources,
                )
                .await
                .unwrap()
        };

        let contentstore_path = ContentStoreAddr::from(output_dir.as_path());
        let (mut build, project) = DataBuildOptions::new_with_sqlite_output(
            output_dir,
            CompilerRegistryOptions::local_compilers(target_dir()),
        )
        .content_store(&contentstore_path)
        .create_with_project(project_dir, repository_index, content_provider)
        .await
        .expect("to create index");

        build.source_pull(&project).await.unwrap();

        let compile_path = ResourcePathId::from(parent_resource).push(refs_asset::RefsAsset::TYPE);
        let manifest = build.compile(compile_path, &test_env()).await.unwrap();

        // both test(child_id) and test(parent_id) are separate resources.
        assert_eq!(manifest.compiled_resources.len(), 2);

        let content_store = HddContentStore::open(contentstore_path).expect("valid content store");
        for checksum in manifest.compiled_resources.iter().map(|a| a.checksum) {
            assert!(content_store.exists(checksum).await);
        }
    }
}
