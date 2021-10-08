fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("./ingestion.proto")?;
    println!("cargo:rerun-if-changed=ingestion.proto");
    Ok(())
}
