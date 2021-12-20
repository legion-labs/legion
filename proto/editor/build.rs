fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "run-codegen")]
    {
        let context = lgn_build_utils::pre_codegen(cfg!(feature = "run-codegen-validation"))?;

        let proto_filepaths = &["./editor.proto"];
        lgn_build_utils::build_protos(
            &context,
            proto_filepaths,
            &["."],
            lgn_build_utils::Language::RUST | lgn_build_utils::Language::TYPESCRIPT,
        )?;

        lgn_build_utils::post_codegen(&context)?;
    }
    Ok(())
}
