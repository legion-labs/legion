#include "crate://lgn-renderer/gpu/pipeline_layout/const_color_pipeline_layout.hlsl"

#include "crate://lgn-renderer/gpu/include/mesh_description.hsh"

struct VertexIn {
    float4 pos : POSITION;
    float4 normal : NORMAL;
    float4 color: COLOR;
    float2 uv_coord : TEXCOORD0;
};

struct VertexOut {  
    float4 hpos : SV_POSITION;
    float4 color: COLOR;
    float2 uv_coord : TEXCOORD0;
};

VertexOut main_vs(uint vertexId: SV_VertexID) {
    MeshDescription mesh_desc = static_buffer.Load<MeshDescription>(push_constant.vertex_offset);

    VertexIn vertex_in = LoadVertex<VertexIn>(mesh_desc, vertexId);
    VertexOut vertex_out;

    float4 pos_view_relative = mul(view_data.view, mul(push_constant.world, vertex_in.pos));
    vertex_out.hpos = mul(view_data.projection, pos_view_relative);
    vertex_out.color = vertex_in.color;
    vertex_out.uv_coord = vertex_in.uv_coord;

    return vertex_out;
}

float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    return saturate(vertex_out.color + push_constant.color);
}
