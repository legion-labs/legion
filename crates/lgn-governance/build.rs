fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=migrations");

    let options = lgn_api_codegen::RustOptions::default();
    lgn_api_codegen::generate!(
        lgn_api_codegen::Language::Rust(options),
        "../../apis",
        ["space", "session", "user", "permission"],
    )?;

    println!("cargo:rerun-if-changed=migrations");

    Ok(())
}
