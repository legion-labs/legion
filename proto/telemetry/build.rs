fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "run-codegen")]
    {
        let context = legion_build_utils::Context::new(cfg!(feature = "run-codegen-validation"));
        legion_build_utils::build_protos(
            &context,
            &["./ingestion.proto", "./analytics.proto"],
            &["."],
            legion_build_utils::Language::RUST | legion_build_utils::Language::TYPESCRIPT,
        )?;

        legion_build_utils::handle_output(&context)?;

        println!("cargo:rerun-if-changed=process.proto");
        println!("cargo:rerun-if-changed=stream.proto");
        println!("cargo:rerun-if-changed=block.proto");
    }
    Ok(())
}
