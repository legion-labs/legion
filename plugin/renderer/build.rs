use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // #[cfg(feature = "run-codegen")]
    {
        let context = lgn_build_utils::pre_codegen(cfg!(feature = "run-codegen-validation"))?;

        let root_cgen = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("root.cgen");
        lgn_build_utils::build_graphics_cgen(&context, &root_cgen)?;

        lgn_build_utils::post_codegen(&context)?;
    }
    Ok(())
}
