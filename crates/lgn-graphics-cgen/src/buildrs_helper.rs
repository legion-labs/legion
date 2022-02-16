use std::path::Path;

use crate::run;

/// .
///
/// # Examples
///
/// ```ignore
/// use lgn_graphics_cgen::buildrs_helper::build_graphics_cgen;
///
/// let result = build_graphics_cgen(crate_name, out_dir, root_file);
/// assert_eq!(result, OK(()));
/// ```
///
/// # Panics
///
/// Panics if .
///
/// # Errors
///
/// This function will return an error if .
///
pub fn run_graphics_cgen(
    crate_name: &str,
    manifest_dir: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    // generate root file name
    let root_file = manifest_dir
        .as_ref()
        .join("gpu")
        .join("codegen")
        .join("root.rn");

    // build context
    let mut ctx_builder = run::CGenContextBuilder::new();
    ctx_builder.set_root_file(root_file).unwrap();
    ctx_builder.set_out_dir(&out_dir).unwrap();
    ctx_builder.set_crate_name(crate_name);

    // run generation
    let result = run::run(&ctx_builder.build());
    match &result {
        Ok(build_result) => {
            for input_dependency in &build_result.input_dependencies {
                println!("cargo:rerun-if-changed={}", input_dependency.display());
            }
        }
        Err(e) => {
            for msg in e.chain() {
                eprintln!("{}", msg);
            }
        }
    }
    result.map(|_| ()).map_err(std::convert::Into::into)
}

/// Run the code generation form a build rs file
/// Expect the variables `CARGO_MANIFEST_DIR` and `OUT_DIR` to be set
/// # Errors
/// Errors on failing to generate build files
pub fn build_graphics_cgen() -> Result<(), Box<dyn std::error::Error>> {
    run_graphics_cgen(
        &std::env::var("CARGO_PKG_NAME").unwrap(),
        Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()),
        Path::new(&std::env::var("OUT_DIR").unwrap()),
    )
    .and_then(|_| lgn_build_utils::symlink_out_dir().map_err(std::convert::Into::into))
}
