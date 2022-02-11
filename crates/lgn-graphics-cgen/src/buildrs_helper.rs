use std::path::Path;

use crate::run;

#[macro_export]
macro_rules! build_graphics_cgen {
    () => {
        run_graphics_cgen(
            &std::env::var("CARGO_PKG_NAME").unwrap(),
            Path::new(env!("CARGO_MANIFEST_DIR")),
            Path::new(&std::env::var("OUT_DIR").unwrap()),
        )
    };
}

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

#[allow(clippy::redundant_closure_for_method_calls)]
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
    let result = result.map(|_| ()).map_err(std::convert::Into::into);

    result.and(if std::env::var("LGN_SYMLINK_OUT_DIR").is_ok() {
        symlink(
            out_dir.as_ref(),
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("out_dir"),
        )
        .map_err(|err| err.into())
    } else {
        Ok(())
    })
}

fn symlink(src: &Path, dst: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    return std::os::windows::fs::symlink_dir(src, dst);
    #[cfg(not(windows))]
    return std::os::unix::fs::symlink(src, dst);
}
