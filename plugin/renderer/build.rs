fn main() -> Result<(), Box<dyn std::error::Error>> {
    // #[cfg(feature = "run-codegen")]
    {
        let crate_folder = env!("CARGO_MANIFEST_DIR").to_string();

        let mut root_cgen = crate_folder.clone();
        root_cgen.push_str("/src/root.cgen");

        let context = lgn_build_utils::Context::new(cfg!(feature = "run-codegen-validation"));
        lgn_build_utils::build_graphics_cgen(&context, &root_cgen)?;

        lgn_build_utils::handle_output(&context)?;
    }
    Ok(())
}
