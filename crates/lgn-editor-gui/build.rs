use cargo_toml::Manifest;
use serde::Deserialize;

#[derive(Deserialize)]
struct NpmMetadata {
    name: String,
}

#[derive(Deserialize)]
struct Metadata {
    npm: NpmMetadata,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "custom-protocol")]
    {
        let cargo: Manifest<Metadata> = Manifest::from_path_with_metadata("cargo.toml")?;

        let npm = cargo
            .package
            .and_then(|p| p.metadata)
            .map(|m| m.npm)
            .ok_or("npm name not found in cargo.toml")?;

        lgn_build_utils_web::build_web_app(&npm.name)?;
    }

    tauri_build::build();
    Ok(())
}
