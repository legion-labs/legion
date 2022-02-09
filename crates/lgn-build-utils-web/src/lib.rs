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

use std::fs;

use cargo_toml::Manifest;
use lgn_build_utils::{Error, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct NpmMetadata {
    name: String,
}

#[derive(Deserialize)]
struct Metadata {
    npm: NpmMetadata,
}

/// Handle the validation of the output files
///
/// # Errors
/// An error is returned only when the client hasn't been built,
/// if the default `index.html` cannot be created or if the `Cargo.toml` cannot be read.
pub fn build_web_app() -> Result<()> {
    // TODO: Should be dynamic based on the metadata in `Cargo.toml`
    if fs::File::open("frontend/dist/index.html").is_err() {
        let cargo: Manifest<Metadata> =
            Manifest::from_path_with_metadata("Cargo.toml").map_err(|error| match error {
                cargo_toml::Error::Io(error) => Error::Io(error),
                cargo_toml::Error::Parse(_error) => Error::Unknown,
            })?;

        let npm = cargo
            .package
            .and_then(|p| p.metadata)
            .map(|m| m.npm)
            .ok_or_else(|| Error::Build("npm name not found in Cargo.toml".into()))?;

        fs::create_dir_all("frontend/dist").map_err(Error::Io)?;

        fs::write(
            "frontend/dist/index.html",
            format!(
                "You need to run `pnpm build {name}` or `cargo m npm build -p {name}`",
                name = npm.name
            ),
        )
        .map_err(Error::Io)?;

        println!("cargo:rerun-if-env-changed=PATH");
        println!("cargo:rerun-if-changed=frontend/dist");
        println!("cargo:rerun-if-changed=frontend/dist/index.html");
    }

    Ok(())
}
