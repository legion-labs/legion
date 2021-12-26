use std::{env, path::PathBuf};

use lgn_data_compiler::{compiler_api::CompilationEnv, Locale, Platform, Target};
use tempfile::TempDir;

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
        .expect("available test directory")
}

pub fn compiler_exe(name: &str) -> PathBuf {
    target_dir().join(format!("compiler-{}{}", name, env::consts::EXE_SUFFIX))
}

pub fn setup_dir(work_dir: &TempDir) -> (PathBuf, PathBuf) {
    let resource_dir = work_dir.path().join("offline");
    let output_dir = work_dir.path().join("temp");

    std::fs::create_dir(&resource_dir).unwrap();
    std::fs::create_dir(&output_dir).unwrap();
    (resource_dir, output_dir)
}

pub fn test_env() -> CompilationEnv {
    CompilationEnv {
        target: Target::Game,
        platform: Platform::Windows,
        locale: Locale::new("en"),
    }
}
