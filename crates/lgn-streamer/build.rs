use lgn_graphics_cgen::buildrs_helper::build_graphics_cgen;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = lgn_api_codegen::RustOptions::default();

    options.add_module_mapping(
        "../lgn-governance/apis/space.yaml",
        "lgn_governance::api::space",
    )?;

    options.add_module_mapping(
        "../lgn-governance/apis/workspace.yaml",
        "lgn_governance::api::workspace",
    )?;

    lgn_api_codegen::generate!(
        lgn_api_codegen::Language::Rust(options),
        "apis",
        ["streaming"]
    );

    build_graphics_cgen()
}
