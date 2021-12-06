fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "run-codegen")]
    {
        let context = lgn_build_utils::Context::new(cfg!(feature = "run-codegen-validation"));
        lgn_build_utils::build_protos(
            &context,
            &["./ingestion.proto", "./analytics.proto"],
            &["."],
            lgn_build_utils::Language::RUST | lgn_build_utils::Language::TYPESCRIPT,
        )?;

        lgn_build_utils::handle_output(&context)?;

        println!("cargo:rerun-if-changed=process.proto");
        println!("cargo:rerun-if-changed=stream.proto");
        println!("cargo:rerun-if-changed=block.proto");
    }
    Ok(())
}
