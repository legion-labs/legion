#include "crate://lgn-graphics-renderer/gpu/pipeline_layout/hzb_pipeline_layout.hlsl"

#include "crate://lgn-graphics-renderer/gpu/include/fullscreen_triangle.hsh"

VertexOut main_vs(in uint id : SV_VERTEXID) {
    return fullscreen_triangle_vertex(id);
}

float main_ps(in VertexOut vertex_out) : SV_TARGET {
    float4 source = depth_texture.Gather(depth_sampler, vertex_out.uv);
    return max(max(source.r, source.g), max(source.b, source.a));
}