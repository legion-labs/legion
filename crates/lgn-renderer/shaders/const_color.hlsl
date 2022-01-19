// #include "crate://renderer/codegen/hlsl/cgen_type/view_data.hlsl"
// #include "crate://renderer/codegen/hlsl/cgen_type/const_data.hlsl"
// #include "crate://renderer/codegen/hlsl/cgen_type/debug_push_constant_data.hlsl"
#include "crate://renderer/codegen/hlsl/pipeline_layout/const_color_pipeline_layout.hlsl"

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


// ConstantBuffer<ViewData> view_data;
// ConstantBuffer<ConstData> const_data;
// ByteAddressBuffer static_buffer;

// [[vk::push_constant]]
// ConstantBuffer<DebugPushConstantData> push_constant;

VertexOut main_vs(uint vertexId: SV_VertexID) {
    VertexIn vertex_in = static_buffer.Load<VertexIn>(push_constant.vertex_offset + vertexId * 56);
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
