fn main() -> Result<(), Box<dyn std::error::Error>> {
    lgn_api_codegen::generate!(
        lgn_api_codegen::Language::Rust(lgn_api_codegen::RustOptions::default()),
        ".",
        ["cars"]
    )?;
    lgn_api_codegen::generate!(lgn_api_codegen::Language::Python, ".", ["cars"])?;

    Ok(())
}
