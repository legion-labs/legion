fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "run-codegen")]
    {
        let proto_filepaths = &[
            "./analytics.proto",
            "./block.proto",
            "./ingestion.proto",
            "./process.proto",
            "./stream.proto",
        ];

        let context = lgn_build_utils::Context::new(cfg!(feature = "run-codegen-validation"));
        lgn_build_utils::build_protos(
            &context,
            proto_filepaths,
            &["."],
            lgn_build_utils::Language::RUST | lgn_build_utils::Language::TYPESCRIPT,
        )?;

        lgn_build_utils::handle_output(&context)?;
    }
    Ok(())
}
