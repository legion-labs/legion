#include "crate://lgn-graphics-renderer/gpu/pipeline_layout/const_color_pipeline_layout.hlsl"

#include "crate://lgn-graphics-renderer/gpu/include/common.hsh"
#include "crate://lgn-graphics-renderer/gpu/include/mesh.hsh"

struct VertexOut {
    float4 hpos : SV_POSITION;
    float4 color: COLOR;
};

VertexOut main_vs(uint vertexId: SV_VertexID) {
    MeshDescription mesh_desc = LoadMeshDescription(static_buffer, push_constant.mesh_description_offset);

    VertexIn vertex_in = LoadVertex<VertexIn>(mesh_desc, vertexId);
    VertexOut vertex_out;

    float3 view_pos = transform_position(view_data, transform_position(push_constant.transform, vertex_in.pos));

    vertex_out.hpos = mul(view_data.projection, float4(view_pos, 1.0));
    vertex_out.color = vertex_in.color;

    return vertex_out;
}

float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    return lerp(vertex_out.color, push_constant.color, push_constant.color_blend); 
}
