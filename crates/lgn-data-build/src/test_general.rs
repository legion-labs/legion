use std::path::PathBuf;

use lgn_content_store::ContentStoreAddr;
use lgn_data_compiler::compiler_node::CompilerRegistryOptions;
use lgn_data_offline::resource::Project;
use tempfile::TempDir;

use crate::{databuild::DataBuild, output_index::OutputIndex, DataBuildOptions, Error};

fn setup_dir(work_dir: &TempDir) -> (PathBuf, PathBuf) {
    let project_dir = work_dir.path();
    let output_dir = project_dir.join("temp");
    std::fs::create_dir(&output_dir).unwrap();
    (project_dir.to_owned(), output_dir)
}

#[tokio::test]
async fn invalid_project() {
    let work_dir = tempfile::tempdir().unwrap();
    let (project_dir, output_dir) = setup_dir(&work_dir);

    let buildindex_dir = output_dir.clone();
    let cas_addr = ContentStoreAddr::from(output_dir);

    let build = DataBuildOptions::new(&buildindex_dir, CompilerRegistryOptions::default())
        .content_store(&cas_addr)
        .create_with_project(&project_dir)
        .await;

    assert!(matches!(build, Err(Error::Project(_))), "{:?}", build);
}

#[tokio::test]
async fn create() {
    let work_dir = tempfile::tempdir().unwrap();
    let (project_dir, output_dir) = setup_dir(&work_dir);

    let _project = Project::create_new(&project_dir)
        .await
        .expect("failed to create a project");

    let buildindex_dir = output_dir.clone();
    let cas_addr = ContentStoreAddr::from(output_dir);

    {
        let _build = DataBuildOptions::new(&buildindex_dir, CompilerRegistryOptions::default())
            .content_store(&cas_addr)
            .create_with_project(project_dir)
            .await
            .expect("valid data build index");
    }

    let _index = OutputIndex::open(OutputIndex::database_uri(
        &buildindex_dir,
        DataBuild::version(),
    ))
    .await
    .expect("failed to open build index file");
}
