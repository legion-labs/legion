#include "crate://lgn-graphics-renderer/gpu/pipeline_layout/final_resolve_pipeline_layout.hlsl"

#include "crate://lgn-graphics-renderer/gpu/include/common.hsh"
#include "crate://lgn-graphics-renderer/gpu/include/fullscreen_triangle.hsh"

VertexOut main_vs(in uint id : SV_VERTEXID) {
    return fullscreen_triangle_vertex(id);
}

float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    float4 hdr_image = linear_texture.Sample(linear_sampler, vertex_out.uv);
    return float4(linear2srgb(tonemap(hdr_image.rgb)), 1.0);
}
