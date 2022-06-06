#include "crate://lgn-presenter-snapshot/gpu/pipeline_layout/display_mapper_pipeline_layout.hlsl"

#include "crate://lgn-graphics-renderer/gpu/include/fullscreen_triangle.hsh"

VertexOut main_vs(uint vertex_id: SV_VERTEXID) {
    return fullscreen_triangle_vertex(vertex_id);
}

float4 main_ps(in VertexOut vertex_out) : SV_TARGET {

    float4 value = hdr_image.Sample(hdr_sampler, vertex_out.uv );

    return value;
}