fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = lgn_api_codegen::RustOptions::default();
    options.add_module_mapping("apis/space.yaml", "lgn_governance::api::space")?;

    lgn_api_codegen::generate!(
        lgn_api_codegen::Language::Rust(options),
        "apis",
        ["content_store"],
    );

    Ok(())
}
