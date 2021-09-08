use hassle_rs::compile_hlsl;

pub fn test_build() {
    let code = "
    Texture2D<float4> g_input    : register(t0, space0);
    RWTexture2D<float4> g_output : register(u0, space1);

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

    let module = rspirv_reflect::Reflection::new_from_spirv(&ir.unwrap()).expect("should work");
    for i in module.get_descriptor_sets().expect("works") {
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
