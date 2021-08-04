use hassle_rs::compile_hlsl;

pub fn test_build() {
    let code = "
    Texture2D<float4> g_input    : register(t0, space0);
    RWTexture2D<float4> g_output : register(u0, space0);

    [numthreads(8, 8, 1)]
    void copyCs(uint3 dispatchThreadId : SV_DispatchThreadID)
    {
        g_output[dispatchThreadId.xy] = g_input[dispatchThreadId.xy];
    }";

    let ir = compile_hlsl(
        "shader_filename.hlsl",
        code,
        "copyCs",
        "cs_6_1",
        &["-spirv"],
        &[("MY_DEFINE", Some("Value")), ("OTHER_DEFINE", None)],
    );

    let module = spirv_reflect::ShaderModule::load_u8_data(&ir.unwrap()).expect("should work");
    for i in module
        .enumerate_descriptor_bindings(Some("copyCs"))
        .expect("works")
    {
        println!("{:?}", i);
    }
}

#[cfg(test)]
mod tests {
    use crate::test_build;

    #[test]
    fn it_works() {
        test_build();

        let four = 2 + 2;
        assert_eq!(four, 4);
    }
}
