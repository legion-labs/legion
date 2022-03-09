#include "crate://lgn-renderer/gpu/pipeline_layout/const_color_pipeline_layout.hlsl"

#include "crate://lgn-renderer/gpu/include/mesh.hsh"

struct VertexIn {
    float3 pos : POSITION;
    float3 normal : NORMAL;
    float3 tangent : TANGENT;
    float4 color: COLOR;
    float2 uv_coord : TEXCOORD0;
};

struct VertexOut {
    float4 hpos : SV_POSITION;
    float4 color: COLOR;
    float2 uv_coord : TEXCOORD0;
};

VertexOut main_vs(uint vertexId: SV_VertexID) {
    MeshDescription mesh_desc = LoadMeshDescription(static_buffer, push_constant.mesh_description_offset);

    VertexIn vertex_in = LoadVertex<VertexIn>(mesh_desc, vertexId);
    VertexOut vertex_out;

    float4 pos_view_relative = mul(view_data.view, mul(push_constant.world, float4(vertex_in.pos, 1.0)));
    vertex_out.hpos = mul(view_data.projection, pos_view_relative);
    vertex_out.color = vertex_in.color;
    vertex_out.uv_coord = vertex_in.uv_coord;

    return vertex_out;
}

float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    return saturate(vertex_out.color + push_constant.color);
}
