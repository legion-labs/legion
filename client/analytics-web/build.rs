fn main() -> Result<(), Box<dyn std::error::Error>> {
    lgn_build_utils::build_web_app()?;
    Ok(())
}
