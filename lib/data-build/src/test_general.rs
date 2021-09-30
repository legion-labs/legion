use std::{fs, path::PathBuf};

use crate::{buildindex::BuildIndex, databuild::DataBuild, DataBuildOptions};
use legion_content_store::ContentStoreAddr;
use legion_data_offline::resource::Project;
use tempfile::TempDir;

pub const TEST_BUILDINDEX_FILENAME: &str = "build.index";

fn setup_dir(work_dir: &TempDir) -> (PathBuf, PathBuf) {
    let project_dir = work_dir.path();
    let output_dir = project_dir.join("temp");
    std::fs::create_dir(&output_dir).unwrap();
    (project_dir.to_owned(), output_dir)
}

#[test]
fn create() {
    let work_dir = tempfile::tempdir().unwrap();
    let (project_dir, output_dir) = setup_dir(&work_dir);

    let projectindex_path = {
        let project = Project::create_new(&project_dir).expect("failed to create a project");
        project.indexfile_path()
    };

    let buildindex_path = output_dir.join(TEST_BUILDINDEX_FILENAME);
    let cas_addr = ContentStoreAddr::from(output_dir);

    {
        let _build = DataBuildOptions::new(&buildindex_path)
            .content_store(&cas_addr)
            .create(project_dir)
            .expect("valid data build index");
    }

    let index = BuildIndex::open(&buildindex_path, DataBuild::version())
        .expect("failed to open build index file");

    assert!(index.validate_project_index());

    fs::remove_file(projectindex_path).unwrap();

    assert!(!index.validate_project_index());
}
