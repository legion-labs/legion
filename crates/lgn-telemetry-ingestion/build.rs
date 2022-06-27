fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = lgn_api_codegen::RustOptions::default();
    options.add_module_mapping(
        "../lgn-governance/apis/space.yaml",
        "lgn_governance::api::space",
    )?;
    options.add_module_mapping(
        "../lgn-telemetry/apis/components.yaml",
        "lgn_telemetry::api::components",
    )?;

    lgn_api_codegen::generate!(
        lgn_api_codegen::Language::Rust(options),
        "apis",
        ["ingestion"]
    );

    Ok(())
}
