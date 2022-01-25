//! Legion build utils
//! This crate is meant to provide helpers for code generation in the monorepo
//! We rely on code generation in multiple instances:
//! * Proto files that generate rust and javascript files
//! * Shader files definition that generate rust and hlsl
//! * Data containers that generate rust files
//!
//! There is 2 ways of handling generated files in the rust ecosystem:
//! * Relying on `OUT_DIR` environment variable to generate in place any
//!   necessary file. (tonic, windows api, ...)
//! * Generating the files in the repo and committing them to the repo.
//!   (rust-analyser, rusoto, ...)
//!
//! We can't generate files in the crate directory and not have them committed,
//! since we have to think about the case of an external dependency being
//! downloaded in the local immutable register.
//!
//! Advantages:
//! * Improves readability and UX of generated files (Go to definition works in
//!   VS Code, looking at code from github)
//! * Allows inclusion of generated files from other systems (Javasctript, hlsl
//!   in a uniform manner) since `OUT_DIR` is only know during the cargo build
//!   of a given crate.
//!
//! Drawbacks:
//! * Dummy conflict in generated code
//! * We lose the ability to modify some src files from the github web interface
//!   since you
//! * Confusion about non generated code and generated code (although mitigated
//!   by conventions)
//!
//! Restriction and rules:
//! * We can't have binary files checked in
//! * Modification of the generated files would not be allowed under any
//!   circumstances, the build machines fail if any change was detected
//! * Files whose generation ca be driven by features, or that are platform
//!   dependent would still use `OUT_DIR`.
//! * Other cases where the in repo generation doesn't bring much

// crate-specific lint exceptions:
//#![allow()]

use lgn_build_utils::{run_cmd, Result};

/// Handle the copy/validation of the output files
///
/// # Errors
/// Returns a generation error or an IO error
pub fn build_web_app() -> Result<()> {
    if let Ok(pnpm_path) = which::which("pnpm") {
        let frontend_dir = "frontend";
        let lock = named_lock::NamedLock::create("pnpm_processing").unwrap();
        let _guard = lock.lock().unwrap();
        run_cmd(&pnpm_path, &["install", "--unsafe-perm"], frontend_dir)?;
        run_cmd(&pnpm_path, &["build"], frontend_dir)?;

        // JS ecosystem forces us to have output files in our sources hierarchy
        // we are filtering files
        std::fs::read_dir(frontend_dir)
            .unwrap()
            .map(|res| res.map(|entry| entry.path()))
            .filter_map(|path| {
                if let Ok(path) = path {
                    if let Some(file_name) = path.file_name() {
                        if file_name != "dist" && file_name != "node_modules" {
                            return Some(path);
                        }
                    }
                }
                None
            })
            .for_each(|path| {
                // to_string_lossy should be fine here, our first level folder names are clean
                println!("cargo:rerun-if-changed={}", path.to_string_lossy());
            });
    } else {
        std::fs::create_dir_all("frontend/dist").unwrap();
        std::fs::write(
            "frontend/dist/index.html",
            "pnpm missing from path, please run a clean build after installing it",
        )
        .unwrap();
        println!("cargo:rerun-if-env-changed=PATH");
    }
    Ok(())
}
