use std::fs::{self};

use crate::{buildindex::BuildIndex, databuild::DataBuild, DataBuildOptions};
use legion_content_store::ContentStoreAddr;
use legion_resources::Project;

pub const TEST_BUILDINDEX_FILENAME: &str = "build.index";

#[test]
fn create() {
    let work_dir = tempfile::tempdir().unwrap();
    let project_dir = work_dir.path();
    let projectindex_path = {
        let project = Project::create_new(project_dir).expect("failed to create a project");
        project.indexfile_path()
    };
    let cas_addr = ContentStoreAddr::from(work_dir.path().to_owned());

    let buildindex_path = project_dir.join(TEST_BUILDINDEX_FILENAME);

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
