#include "crate://renderer/codegen/hlsl/pipeline_layout/egui_pipeline_layout.hlsl"

struct VertexIn {
    float2 pos : POSITION;
    float2 uv : TEXCOORD;
    float4 color : COLOR;
};

struct VertexOut {  
    float4 hpos : SV_POSITION;
    float2 uv : TEXCOORD;
    float4 color : COLOR;
};

// See https://github.com/emilk/egui/blob/26d576f5101dfa1219f79bf9c99e29c577487cd3/egui_glium/src/painter.rs#L19.
float3 linear_from_srgb(float3 srgb) {
    bool3 cutoff = srgb < float3(10.31475, 10.31475, 10.31475);
    float3 lower = srgb / float3(3294.6, 3294.6, 3294.6);
    float3 higher = pow((srgb + float3(14.025, 14.025, 14.025)) / float3(269.025, 269.025, 269.025), float3(2.4, 2.4, 2.4));
    return lerp(higher, lower, cutoff);
}
float4 linear_from_srgba(float4 srgba) {
    return float4(linear_from_srgb(srgba.rgb), srgba.a / 255.0);
}

VertexOut main_vs(in VertexIn vertex_in) {
    VertexOut vertex_out;
    vertex_out.hpos = float4(float2(
        2*vertex_in.pos.x/push_constant.width - 1.0,
        1.0 - 2*vertex_in.pos.y/push_constant.height) * push_constant.scale + push_constant.translation, 0.0, 1.0);
    vertex_out.uv = vertex_in.uv;
    vertex_out.color = linear_from_srgba(vertex_in.color); //TODO: sRGB
    return vertex_out;
}

float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    return vertex_out.color*font_texture.Sample(font_sampler, vertex_out.uv).r;
}