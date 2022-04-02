#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Arc};

    use lgn_content_store::ContentStoreAddr;
    use lgn_content_store2::{ContentProvider, MemoryProvider};
    use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
    use lgn_data_offline::resource::Project;
    use lgn_source_control::LocalRepositoryIndex;
    use tempfile::TempDir;

    use crate::{databuild::DataBuild, output_index::OutputIndex, DataBuildOptions, Error};

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

    #[tokio::test]
    async fn invalid_project() {
        let work_dir = tempfile::tempdir().unwrap();
        let (project_dir, output_dir, repository_index, content_provider) =
            setup_dir(&work_dir).await;

        let cas_addr = ContentStoreAddr::from(output_dir.clone());

        let build = DataBuildOptions::new_with_sqlite_output(
            &output_dir,
            CompilerRegistryOptions::default(),
        )
        .content_store(&cas_addr)
        .create_with_project(&project_dir, repository_index, content_provider)
        .await;

        assert!(matches!(build, Err(Error::Project(_))), "{:?}", build);
    }

    #[tokio::test]
    async fn create() {
        let work_dir = tempfile::tempdir().unwrap();
        let (project_dir, output_dir, repository_index, content_provider) =
            setup_dir(&work_dir).await;

        let _project =
            Project::create_with_remote_mock(&project_dir, Arc::clone(&content_provider))
                .await
                .expect("failed to create a project");

        let cas_addr = ContentStoreAddr::from(output_dir.clone());

        let db_uri =
            DataBuildOptions::output_db_path_dir(output_dir, &project_dir, DataBuild::version());

        {
            let _build =
                DataBuildOptions::new(db_uri.clone(), cas_addr, CompilerRegistryOptions::default())
                    .create_with_project(project_dir, repository_index, content_provider)
                    .await
                    .expect("valid data build index");
        }

        let _index = OutputIndex::open(db_uri)
            .await
            .expect("failed to open build index file");
    }
}
