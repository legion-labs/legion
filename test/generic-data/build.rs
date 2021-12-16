fn main() {
    #[cfg(feature = "run-codegen")]
    {
        let cargo_path = env!("CARGO_MANIFEST_DIR").to_lowercase();
        let directory = std::path::Path::new(&cargo_path).join("def");
        lgn_data_codegen::generate_for_directory(&directory).unwrap();
        println!("cargo:rerun-if-changed={}", directory.display());
    }

    #[cfg(not(feature = "run-codegen"))]
    {
        println!("cargo:rerun-if-changed=build.rs");
    }
}
