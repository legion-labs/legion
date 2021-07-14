pub fn compile_hlsl() {
    let source = "
    Texture2D<float4> g_input    : register(t0, space0);
    RWTexture2D<float4> g_output : register(u0, space0);
    
    [numthreads(8, 8, 1)]
    void copyCs(uint3 dispatchThreadId : SV_DispatchThreadID)
    {
        g_output[dispatchThreadId.xy] = g_input[dispatchThreadId.xy];
    }
    ";

    let mut compiler = shaderc::Compiler::new().unwrap();
    let mut options = shaderc::CompileOptions::new().unwrap();
    options.add_macro_definition("EP", Some("main"));
    options.set_source_language(shaderc::SourceLanguage::HLSL);
    let binary_result = compiler
        .compile_into_spirv(
            source,
            shaderc::ShaderKind::Compute,
            "shader.hlsl",
            "copyCs",
            Some(&options),
        )
        .unwrap();

    assert_eq!(Some(&0x07230203), binary_result.as_binary().first());

    let module = spirv_reflect::ShaderModule::load_u8_data(binary_result.as_binary_u8())
        .expect("should work");
    for i in module
        .enumerate_descriptor_bindings(Some("copyCs"))
        .expect("works")
    {
        println!("{:?}", i);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let four = 2 + 2;
        assert_eq!(four, 4);
    }
}
