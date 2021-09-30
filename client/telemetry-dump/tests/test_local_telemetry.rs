use std::path::{Path, PathBuf};
use test_utils::*;

static DUMP_EXE_VAR: &str = env!("CARGO_BIN_EXE_telemetry-dump");

fn test_dir(test_name: &str) -> PathBuf {
    let parent = Path::new(DUMP_EXE_VAR)
        .parent()
        .unwrap()
        .join("telemetry-dump");
    create_test_dir(&parent, test_name)
}

#[test]
fn test_local_dump() {
    let src_dir = std::env::current_dir().unwrap().join("tests/data");
    let test_output = test_dir("list-processes");
    fs_extra::dir::copy(&src_dir, &test_output, &fs_extra::dir::CopyOptions::new()).unwrap();
}
