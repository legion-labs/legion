fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "custom-protocol")]
    legion_build_utils::build_web_app()?;

    tauri_build::build();
    Ok(())
}
