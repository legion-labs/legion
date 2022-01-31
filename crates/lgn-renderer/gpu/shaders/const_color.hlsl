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

    if (HasIndex(mesh_desc.format))
    {
        vertexId = static_buffer.Load<uint>(mesh_desc.index_offset + vertexId * 4);
    }
    VertexIn vertex_in;
    if (HasPosition(mesh_desc.format))
    {
        vertex_in.pos = static_buffer.Load<float4>(mesh_desc.position_offset + vertexId * 16);
    }
    if (HasNormal(mesh_desc.format))
    {
        vertex_in.normal = static_buffer.Load<float4>(mesh_desc.normal_offset + vertexId * 16);
    }
    if (HasColor(mesh_desc.format))
    {
        vertex_in.color = static_buffer.Load<float4>(mesh_desc.color_offset + vertexId * 16);
    }
    if (HasTexCoord(mesh_desc.format))
    {
        vertex_in.uv_coord = static_buffer.Load<float2>(mesh_desc.tex_coord_offset + vertexId * 8);
    }
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
