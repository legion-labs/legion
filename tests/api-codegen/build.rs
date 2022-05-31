fn main() -> Result<(), Box<dyn std::error::Error>> {
    lgn_api_codegen::generate!(Rust, ".", ["cars"])?;
    lgn_api_codegen::generate!(Python, ".", ["cars"])?;

    Ok(())
}
