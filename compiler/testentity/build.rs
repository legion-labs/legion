fn main() {
    let source =
        legion_data_codegen::definition_path("../../test/generic_data_offline/test_entity.rs");
    legion_data_codegen::generate_data_compiler_code(&source).expect("Compiler codegen failed");
    println!("cargo:rerun-if-changed={:?}", source);
}
