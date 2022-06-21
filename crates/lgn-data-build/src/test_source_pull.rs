#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Arc};

    use generic_data::offline::TestResource;
    use lgn_content_store::Provider;
    use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
    use lgn_data_offline::{Project, SourceResource};
    use lgn_data_runtime::prelude::*;
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
        let mut options = AssetRegistryOptions::new();
        generic_data::register_types(&mut options);
        options.create().await
    }

    #[tokio::test]
    async fn no_dependencies() {
        let work_dir = tempfile::tempdir().unwrap();
        let (project_dir, output_dir, source_control_content_provider, data_content_provider) =
            setup_dir(&work_dir);
        let _resources = setup_registry().await;

        let mut project = Project::new_with_remote_mock(
            &project_dir,
            Arc::clone(&source_control_content_provider),
        )
        .await
        .expect("failed to create a project");

        let resource = ResourcePathId::from({
            let resource_id = project
                .add_resource(&TestResource::new_named("resource"))
                .await
                .unwrap();
            resource_id
        });

        let mut build = DataBuildOptions::new_with_sqlite_output(
            &output_dir,
            CompilerRegistryOptions::default(),
            Arc::clone(&source_control_content_provider),
            data_content_provider,
        )
        .create(&project)
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
        let _resources = setup_registry().await;

        let mut project = Project::new_with_remote_mock(
            &project_dir,
            Arc::clone(&source_control_content_provider),
        )
        .await
        .expect("failed to create a project");

        let (child_id, parent_id) = {
            let child_id = project
                .add_resource(&TestResource::new_named("child"))
                .await
                .unwrap();

            let mut parent = TestResource::new_named("parent");
            parent.build_deps.push(ResourcePathId::from(child_id));

            let parent_id = project.add_resource(&parent).await.unwrap();

            (
                ResourcePathId::from(child_id),
                ResourcePathId::from(parent_id),
            )
        };

        let mut build = DataBuildOptions::new_with_sqlite_output(
            &output_dir,
            CompilerRegistryOptions::default(),
            Arc::clone(&source_control_content_provider),
            data_content_provider,
        )
        .create(&project)
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
