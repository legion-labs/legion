fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(&["./ingestion.proto"], &["./"])?;
    println!("cargo:rerun-if-changed=ingestion.proto");
    println!("cargo:rerun-if-changed=process.proto");
    println!("cargo:rerun-if-changed=stream.proto");
    println!("cargo:rerun-if-changed=block.proto");
    Ok(())
}
