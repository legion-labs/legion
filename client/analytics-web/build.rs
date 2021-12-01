fn main() -> Result<(), Box<dyn std::error::Error>> {
    legion_build_utils::build_web_app()?;
    Ok(())
}
