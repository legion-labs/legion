fn main() {
    let definition_path = "../../test/generic_data_offline/debug_cube.rs";
    let package_path = env!("CARGO_MANIFEST_DIR").to_lowercase();
    let data_path = std::path::Path::new(&package_path);
    let data_path = data_path.join(definition_path);
    legion_data_codegen::generate_data_compiler_code(&data_path).expect("Compiler codegen failed");
    println!("cargo:rerun-if-changed={:?}", data_path);
}
