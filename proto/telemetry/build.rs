#[allow(clippy::unnecessary_wraps)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "run-codegen")]
    {
        let context = lgn_build_utils::pre_codegen(cfg!(feature = "run-codegen-validation"))?;

        let proto_filepaths = &[
            "./analytics.proto",
            "./block.proto",
            "./calltree.proto",
            "./ingestion.proto",
            "./process.proto",
            "./stream.proto",
        ];

        lgn_build_utils_proto::build_protos(
            &context,
            proto_filepaths,
            &["."],
            lgn_build_utils::Language::RUST | lgn_build_utils::Language::TYPESCRIPT,
        )?;

        lgn_build_utils::post_codegen(&context)?;
    }
    Ok(())
}
