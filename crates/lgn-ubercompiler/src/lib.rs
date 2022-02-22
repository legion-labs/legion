use lgn_data_compiler::compiler_node;

pub fn create() -> compiler_node::CompilerRegistryOptions {
    let registry = compiler_node::CompilerRegistryOptions::default();
    registry
        .add_compiler(&lgn_compiler_material::COMPILER_INFO)
        .add_compiler(&lgn_compiler_png2tex::COMPILER_INFO)
        .add_compiler(&lgn_compiler_psd2tex::COMPILER_INFO)
        .add_compiler(&lgn_compiler_runtime_entity::COMPILER_INFO)
        .add_compiler(&lgn_compiler_runtime_instance::COMPILER_INFO)
        .add_compiler(&lgn_compiler_runtime_model::COMPILER_INFO)
        .add_compiler(&lgn_compiler_test_atoi::COMPILER_INFO)
        .add_compiler(&lgn_compiler_test_base64::COMPILER_INFO)
        .add_compiler(&lgn_compiler_test_refs::COMPILER_INFO)
        .add_compiler(&lgn_compiler_test_reverse::COMPILER_INFO)
        .add_compiler(&lgn_compiler_test_split::COMPILER_INFO)
        .add_compiler(&lgn_compiler_testentity::COMPILER_INFO)
        .add_compiler(&lgn_compiler_tex2bin::COMPILER_INFO)
        .add_compiler(&lgn_compiler_script2asm::COMPILER_INFO)
        .add_compiler(&lgn_compiler_gltf::COMPILER_INFO)
}
