use std::{
    env,
    path::{Path, PathBuf},
};

use legion_data_compiler::Manifest;

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
        .unwrap_or_else(|| panic!("cannot find test directory"))
}

pub fn data_build_exe() -> PathBuf {
    target_dir().join(format!("data-build{}", env::consts::EXE_SUFFIX))
}

fn exec_create_build_index(
    path: impl AsRef<Path>,
    project: impl AsRef<Path>,
) -> std::io::Result<std::process::Output> {
    let mut command = std::process::Command::new(data_build_exe());
    command.arg("create");
    command.arg(path.as_ref().to_str().unwrap());
    command.arg(format!("--project={}", project.as_ref().to_str().unwrap()));
    let output = command.output()?;
    assert!(output.status.success());
    Ok(output)
}

fn exec_data_compile(
    compile_path: &str,
    buildindex_path: &Path,
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
    command.arg(format!(
        "--buildindex={}",
        buildindex_path.to_str().unwrap()
    ));
    let output = command.output()?;
    assert!(output.status.success());
    Ok(output)
}

/// read the build index and remove non-deterministic data.
fn read_build_index(buildindex_path: &Path) -> String {
    let content = std::fs::read_to_string(&buildindex_path).expect("file content");
    let mut content = json::parse(&content).expect("valid json");
    content.remove("project_index");
    content.pretty(2)
}

#[test]
fn incremental_build() {
    let work_dir = tempfile::tempdir().unwrap();
    let sampledata_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project = sampledata_dir.join("project.index");
    let buildindex = work_dir.as_ref().join("build.index");

    // create build index and do a source pull
    exec_create_build_index(&buildindex, project).expect("new build index");

    insta::assert_snapshot!("initial_index", read_build_index(&buildindex));

    // default root object in sample data
    // /world/sample_1.ent (offline_entity) => runtime_entity
    let root_entity = "d004cd1c00000000fcd3242ec9691beb|019c8223";

    //
    // first data build
    //
    let out =
        exec_data_compile(root_entity, &buildindex, work_dir.path()).expect("build completed");

    let manifest: Manifest = serde_json::from_slice(&out.stdout).expect("valid manifest");
    insta::assert_json_snapshot!("first_manifest", manifest);

    let first_buildindex = read_build_index(&buildindex);
    insta::assert_snapshot!("first_index", first_buildindex);

    //
    // incremental data build
    //
    let out =
        exec_data_compile(root_entity, &buildindex, work_dir.path()).expect("build completed");

    let incremental_manifest: Manifest =
        serde_json::from_slice(&out.stdout).expect("valid manifest");

    insta::assert_json_snapshot!("incremental_manifest", incremental_manifest);

    let incremental_buildindex = read_build_index(&buildindex);
    insta::assert_snapshot!("incremental_index", incremental_buildindex);

    // incremental build with no changes should not affect the build index
    assert_eq!(first_buildindex, incremental_buildindex);
}
