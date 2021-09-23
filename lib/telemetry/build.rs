fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("./telemetry_ingestion.proto")?;
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=telemetry_ingestion.proto");
    Ok(())
}
