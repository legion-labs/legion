#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Arc};

    use lgn_content_store::Provider;
    use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
    use lgn_data_offline::resource::Project;
    use lgn_source_control::{BranchName, LocalRepositoryIndex, RepositoryName};
    use tempfile::TempDir;

    use crate::{databuild::DataBuild, output_index::OutputIndex, DataBuildOptions};

    pub(crate) async fn setup_dir(
        work_dir: &TempDir,
    ) -> (
        PathBuf,
        PathBuf,
        LocalRepositoryIndex,
        Arc<Provider>,
        Arc<Provider>,
    ) {
        let project_dir = work_dir.path();
        let output_dir = project_dir.join("temp");
        std::fs::create_dir_all(&output_dir).unwrap();

        let repository_index = LocalRepositoryIndex::new(project_dir.join("remote"))
            .await
            .unwrap();
        let source_control_content_provider = Arc::new(Provider::new_in_memory());
        let data_content_provider = Arc::new(Provider::new_in_memory());

        (
            project_dir.to_owned(),
            output_dir,
            repository_index,
            source_control_content_provider,
            data_content_provider,
        )
    }

    #[tokio::test]
    async fn invalid_project() {
        let work_dir = tempfile::tempdir().unwrap();
        let (
            _project_dir,
            output_dir,
            repository_index,
            source_control_content_provider,
            data_content_provider,
        ) = setup_dir(&work_dir).await;

        let repository_name: RepositoryName = "default".parse().unwrap();
        let branch_name: BranchName = "main".parse().unwrap();

        let project = Project::new(
            repository_index,
            &repository_name,
            &branch_name,
            Arc::clone(&source_control_content_provider),
            Arc::clone(&data_content_provider),
        )
        .await;

        if let Ok(project) = project {
            let build = DataBuildOptions::new_with_sqlite_output(
                &output_dir,
                CompilerRegistryOptions::default(),
                Arc::clone(&source_control_content_provider),
                data_content_provider,
            )
            .create(&project)
            .await;

            assert!(build.is_err());
        }
    }

    #[tokio::test]
    async fn create() {
        let work_dir = tempfile::tempdir().unwrap();
        let (
            project_dir,
            output_dir,
            _repository_index,
            source_control_content_provider,
            data_content_provider,
        ) = setup_dir(&work_dir).await;

        let project = Project::new_with_remote_mock(
            &project_dir,
            Arc::clone(&source_control_content_provider),
            Arc::clone(&data_content_provider),
        )
        .await
        .expect("failed to create a project");

        let db_uri =
            DataBuildOptions::output_db_path_dir(output_dir, &project_dir, DataBuild::version());

        {
            let _build = DataBuildOptions::new(
                db_uri.clone(),
                source_control_content_provider,
                data_content_provider,
                CompilerRegistryOptions::default(),
            )
            .create(&project)
            .await
            .expect("valid data build index");
        }

        let _index = OutputIndex::open(db_uri)
            .await
            .expect("failed to open build index file");
    }
}
