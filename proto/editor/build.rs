fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "run-codegen")]
    {
        let context = legion_build_utils::Context::new(cfg!(feature = "run-codegen-validation"));
        legion_build_utils::build_protos(
            &context,
            &["./editor.proto"],
            &["."],
            legion_build_utils::Language::RUST | legion_build_utils::Language::TYPESCRIPT,
        )?;

        legion_build_utils::handle_output(&context)?;
    }
    Ok(())
}
