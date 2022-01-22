fn main() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("def");
    lgn_data_codegen::generate_for_directory(&directory, std::env::var("OUT_DIR").unwrap())
        .unwrap();
    println!("cargo:rerun-if-changed={}", directory.display());
}
