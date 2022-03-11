#include "crate://lgn-graphics-renderer/gpu/pipeline_layout/hzb_pipeline_layout.hlsl"

struct VertexIn {
    float2 pos : POSITION;
    float2 uv : TEXCOORD;
};

struct VertexOut {  
    float4 hpos : SV_POSITION;
    float2 uv : TEXCOORD;
};

VertexOut main_vs(in VertexIn vertex_in) {
    VertexOut vertex_out;

    vertex_out.hpos = float4(float2(2.0 * vertex_in.pos.x - 1.0, 1.0 - 2.0 * vertex_in.pos.y), 0.0, 1.0);
    vertex_out.uv = vertex_in.uv;

    return vertex_out;
}

float main_ps(in VertexOut vertex_out) : SV_TARGET {
    float4 source = depth_texture.Gather(depth_sampler, vertex_out.uv);
    return max(max(source.r, source.g), max(source.b, source.a));
}