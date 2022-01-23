use std::path::{Path, PathBuf};

use lgn_build_utils::{Context, Error};

// TODO: Put this in lgn_graphics_cgen or in it's own lib
fn build_graphics_cgen(
    context: &Context,
    root_file: &impl AsRef<Path>,
) -> lgn_build_utils::Result<()> {
    // build context
    let out_dir = PathBuf::from(&context.codegen_out_dir());
    let mut ctx_builder = lgn_graphics_cgen::run::CGenContextBuilder::new();
    ctx_builder.set_root_file(root_file).unwrap();
    ctx_builder.set_outdir(&out_dir).unwrap();
    //ctx_builder.set_crate_name(std::env::var("CARGO_PKG_NAME").unwrap());
    ctx_builder.set_crate_name("renderer");

    // run generation
    let result = lgn_graphics_cgen::run::run(&ctx_builder.build());
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
    result
        .map(|_| ())
        .map_err(|e| Error::Build(format!("{:?}", e)))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // #[cfg(feature = "run-codegen")]
    {
        let context = lgn_build_utils::pre_codegen(cfg!(feature = "run-codegen-validation"))?;

        let root_cgen = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("root.cgen");
        build_graphics_cgen(&context, &root_cgen)?;

        lgn_build_utils::post_codegen(&context)?;
    }
    Ok(())
}
