#include "crate://lgn-graphics-renderer/gpu/pipeline_layout/egui_pipeline_layout.hlsl"

#include "crate://lgn-graphics-renderer/gpu/include/common.hsh"

struct VertexInUi {
    float2 pos : POSITION;
    float2 uv : TEXCOORD;
    uint color : COLOR;
};

struct VertexOut {  
    float4 hpos : SV_POSITION;
    float2 uv : TEXCOORD;
    float4 color : COLOR;
};

VertexOut main_vs(in VertexInUi vertex_in) {
    VertexOut vertex_out;
    vertex_out.hpos = float4(float2(
        2*vertex_in.pos.x/push_constant.width - 1.0,
        1.0 - 2*vertex_in.pos.y/push_constant.height) * push_constant.scale + push_constant.translation, 0.0, 1.0);
    vertex_out.uv = vertex_in.uv;
    vertex_out.color = unpack_srgb2linear( vertex_in.color  );
    return vertex_out;
}

float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    // 1) vertex_out.color is interpolated in linear space
    // 2) font_texture is automatically transformed to linear space when sampling
    // = output is in linear space and will be gamma corrected in the final resolve
    return vertex_out.color * font_texture.Sample(font_sampler, vertex_out.uv).r ;
}