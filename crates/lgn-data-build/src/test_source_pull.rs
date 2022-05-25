#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Arc};

    use lgn_content_store::Provider;
    use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
    use lgn_data_offline::resource::{Project, ResourcePathName};
    use lgn_data_runtime::{
        AssetRegistry, AssetRegistryOptions, ResourceDescriptor, ResourcePathId,
    };
    use tempfile::TempDir;

    use crate::DataBuildOptions;

    pub(crate) fn setup_dir(
        work_dir: &TempDir,
    ) -> (PathBuf, PathBuf, Arc<Provider>, Arc<Provider>) {
        let project_dir = work_dir.path();
        let output_dir = project_dir.join("temp");
        std::fs::create_dir_all(&output_dir).unwrap();

        let source_control_content_provider = Arc::new(Provider::new_in_memory());
        let data_content_provider = Arc::new(Provider::new_in_memory());

        (
            project_dir.to_owned(),
            output_dir,
            source_control_content_provider,
            data_content_provider,
        )
    }

    async fn setup_registry() -> Arc<AssetRegistry> {
        AssetRegistryOptions::new()
            .add_processor::<refs_resource::TestResource>()
            .create()
            .await
    }

    #[tokio::test]
    async fn no_dependencies() {
        let work_dir = tempfile::tempdir().unwrap();
        let (project_dir, output_dir, source_control_content_provider, data_content_provider) =
            setup_dir(&work_dir);
        let resources = setup_registry().await;

        let mut project =
            Project::create_with_remote_mock(&project_dir, source_control_content_provider)
                .await
                .expect("failed to create a project");

        let resource = ResourcePathId::from(
            project
                .add_resource(
                    ResourcePathName::new("resource"),
                    refs_resource::TestResource::TYPENAME,
                    refs_resource::TestResource::TYPE,
                    &resources
                        .new_resource(refs_resource::TestResource::TYPE)
                        .unwrap(),
                    &resources,
                )
                .await
                .unwrap(),
        );

        let mut build = DataBuildOptions::new_with_sqlite_output(
            &output_dir,
            CompilerRegistryOptions::default(),
            data_content_provider,
        )
        .create()
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
        let (project_dir, output_dir, source_control_content_provider, data_content_provider) =
            setup_dir(&work_dir);
        let resources = setup_registry().await;

        let mut project =
            Project::create_with_remote_mock(&project_dir, source_control_content_provider)
                .await
                .expect("failed to create a project");

        let (child_id, parent_id) = {
            let child_id = project
                .add_resource(
                    ResourcePathName::new("child"),
                    refs_resource::TestResource::TYPENAME,
                    refs_resource::TestResource::TYPE,
                    &resources
                        .new_resource(refs_resource::TestResource::TYPE)
                        .unwrap(),
                    &resources,
                )
                .await
                .unwrap();

            let parent_handle = {
                let res = resources
                    .new_resource(refs_resource::TestResource::TYPE)
                    .unwrap();
                let mut edit = res
                    .instantiate::<refs_resource::TestResource>(&resources)
                    .unwrap();
                edit.build_deps.push(ResourcePathId::from(child_id));
                res.apply(edit, &resources);
                res
            };
            let parent_id = project
                .add_resource(
                    ResourcePathName::new("parent"),
                    refs_resource::TestResource::TYPENAME,
                    refs_resource::TestResource::TYPE,
                    &parent_handle,
                    &resources,
                )
                .await
                .unwrap();

            (
                ResourcePathId::from(child_id),
                ResourcePathId::from(parent_id),
            )
        };

        let mut build = DataBuildOptions::new_with_sqlite_output(
            &output_dir,
            CompilerRegistryOptions::default(),
            data_content_provider,
        )
        .create()
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
}
