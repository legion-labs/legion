use lgn_api_codegen::Language;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let in_file = "./cars.yaml";
    let out_dir = std::env::var("OUT_DIR")?;
    lgn_api_codegen::generate(Language::Rust, in_file, out_dir)?;

    println!("cargo:rerun-if-changed={}", in_file);

    Ok(())
}
