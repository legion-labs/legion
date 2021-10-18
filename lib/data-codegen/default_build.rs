fn main() -> Result<(), Box<dyn std::error::Error>> {
    let package_path = env!("CARGO_MANIFEST_DIR").to_lowercase();

    let mut data_path = package_path.replace("_runtime", "_offline");
    data_path.push_str("/data.rs");

    if package_path.ends_with("_offline") {
        legion_data_codegen::generate_data_container_code(
            std::path::Path::new(&data_path),
            &legion_data_codegen::GenerationType::OfflineFormat,
        )?;
    } else if package_path.ends_with("_runtime") {
        legion_data_codegen::generate_data_container_code(
            std::path::Path::new(&data_path),
            &legion_data_codegen::GenerationType::RuntimeFormat,
        )?;
    }
    println!("cargo:rerun-if-changed={}", data_path);
    Ok(())
}
