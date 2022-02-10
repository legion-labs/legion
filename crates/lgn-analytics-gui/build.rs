fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "custom-protocol")]
    lgn_build_utils_web::build_web_app()?;

    tauri_build::build();

    Ok(())
}
