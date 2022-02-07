use std::{
    env,
    path::{Path, PathBuf},
};

use lgn_data_compiler::CompiledResources;

pub fn target_dir() -> PathBuf {
    env::current_exe()
        .ok()
        .map(|mut path| {
            path.pop();
            if path.ends_with("deps") {
                path.pop();
            }
            path
        })
        .expect("a test directory")
}

#[must_use]
pub fn data_build_exe() -> PathBuf {
    target_dir().join(format!("data-build{}", env::consts::EXE_SUFFIX))
}

fn exec_create_build_index(
    path: impl AsRef<Path>,
    project: impl AsRef<Path>,
    cas: impl AsRef<Path>,
) -> std::io::Result<std::process::Output> {
    let mut command = std::process::Command::new(data_build_exe());
    command.arg("create");
    command.arg(path.as_ref().to_str().unwrap());
    command.arg(format!("--project={}", project.as_ref().to_str().unwrap()));
    command.arg(format!("--cas={}", cas.as_ref().to_str().unwrap()));
    let output = command.output()?;
    assert!(output.status.success());
    Ok(output)
}

fn exec_data_compile(
    compile_path: &str,
    buildindex_dir: &Path,
    project: &Path,
    destination: &Path,
) -> std::io::Result<std::process::Output> {
    let target = "game";
    let platform = "windows";
    let locale = "en";
    let mut command = std::process::Command::new(data_build_exe());
    command.arg("compile");
    command.arg(compile_path);
    command.arg(format!("--cas={}", destination.to_str().unwrap()));
    command.arg(format!("--target={}", target));
    command.arg(format!("--platform={}", platform));
    command.arg(format!("--locale={}", locale));
    command.arg(format!("--buildindex={}", buildindex_dir.to_str().unwrap()));
    command.arg(format!("--project={}", project.to_str().unwrap()));
    let output = command.output()?;
    if !output.status.success() {
        println!("'{}'", std::str::from_utf8(&output.stdout).unwrap());
        println!("'{}'", std::str::from_utf8(&output.stderr).unwrap());
    }
    assert!(output.status.success());
    Ok(output)
}

/// read the build index and remove non-deterministic data.
fn read_build_output(buildindex_dir: &Path) -> String {
    let content =
        std::fs::read_to_string(&buildindex_dir.join("output.index")).expect("file content");
    let mut content = json::parse(&content).expect("valid json");
    content.remove("project_index");
    content.pretty(2)
}

#[test]
fn incremental_build() {
    let work_dir = tempfile::tempdir().unwrap();
    let sampledata_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project = sampledata_dir.join("project.index");
    let buildindex = work_dir.as_ref().to_owned();
    let temp_dir = work_dir.path().join("temp");
    std::fs::create_dir(&temp_dir).unwrap();

    // create build index and do a source pull
    exec_create_build_index(&buildindex, &project, &temp_dir).expect("new build index");

    insta::assert_snapshot!("initial_index", read_build_output(&buildindex));

    // default root object in sample data
    // /world/sample_1.ent (offline_entity) => runtime_entity
    let root_entity = "(1c0ff9e497b0740f,5d6c8521-ef7f-4402-8fac-01c5c4f53329)|1d9ddd99aad89045";

    //
    // first data build
    //
    let out =
        exec_data_compile(root_entity, &buildindex, &project, &temp_dir).expect("build completed");

    let manifest: CompiledResources = serde_json::from_slice(&out.stdout).expect("valid manifest");
    insta::assert_json_snapshot!("first_manifest", manifest);

    let first_buildindex = read_build_output(&buildindex);
    insta::assert_snapshot!("first_index", first_buildindex);

    //
    // incremental data build
    //
    let out =
        exec_data_compile(root_entity, &buildindex, &project, &temp_dir).expect("build completed");

    let incremental_manifest: CompiledResources =
        serde_json::from_slice(&out.stdout).expect("valid manifest");

    insta::assert_json_snapshot!("incremental_manifest", incremental_manifest);

    let incremental_buildindex = read_build_output(&buildindex);
    insta::assert_snapshot!("incremental_index", incremental_buildindex);

    // incremental build with no changes should not affect the build index
    assert_eq!(first_buildindex, incremental_buildindex);
}
