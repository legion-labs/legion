fn main() {
    let source = lgn_data_codegen::definition_path("../../test/generic_data_offline/debug_cube.rs");
    lgn_data_codegen::generate_data_compiler_code(&source).expect("Compiler codegen failed");
    println!("cargo:rerun-if-changed={}", source.to_str().unwrap());
}
