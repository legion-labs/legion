fn main() -> Result<(), Box<dyn std::error::Error>> {
    lgn_api_codegen::generate!(Rust, "../../apis", governance, session)?;

    Ok(())
}
