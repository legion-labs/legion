fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=migrations");

    lgn_api_codegen::generate!(Rust, "../../apis", ["space", "session", "user"])?;

    Ok(())
}
