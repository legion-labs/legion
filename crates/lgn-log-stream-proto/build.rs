fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR")?;
    let protos = &["./protos/log_stream.proto"];
    tonic_build::configure()
        .out_dir(&out_dir)
        .compile(protos, &["."])?;

    for proto in protos {
        println!("cargo:rerun-if-changed={}", proto);
    }
    Ok(())
}
