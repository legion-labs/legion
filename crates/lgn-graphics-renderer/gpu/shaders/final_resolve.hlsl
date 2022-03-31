#include "crate://lgn-graphics-renderer/gpu/pipeline_layout/final_resolve_pipeline_layout.hlsl"

#include "crate://lgn-graphics-renderer/gpu/include/common.hsh"

struct VertexOut {  
    float4 hpos : SV_POSITION;
    float2 uv : TEXCOORD;
};

VertexOut main_vs(in uint vertexId : SV_VertexID) {
    VertexOut vertex_out;

    if (vertexId == 0) {
        vertex_out.hpos = float4(-1.0, -3.0, 0.0, 1.0);
        vertex_out.uv = float2(0.0, 2.0);
    }
    if (vertexId == 1) {
        vertex_out.hpos = float4(3.0, 1.0, 0.0, 1.0);
        vertex_out.uv = float2(2.0, 0.0);
    }
    if (vertexId == 2) {
        vertex_out.hpos = float4(-1.0, 1.0, 0.0, 1.0);
        vertex_out.uv = float2(0.0, 0.0);
    }

    return vertex_out;
}

float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    float4 hdr_image = linear_texture.Sample(linear_sampler, vertex_out.uv);
    return float4(linear2srgb(tonemap(hdr_image.rgb)), 1.0);
}
