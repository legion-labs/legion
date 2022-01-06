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

struct ConstData {
    float4x4 world;
    float4x4 view;
    float4x4 projection;
    float4 color;
};

ConstantBuffer<ConstData> const_data;
ByteAddressBuffer static_buffer;

struct PushConstData {
    uint vertex_offset;
};

[[vk::push_constant]]
ConstantBuffer<PushConstData> push_constant;

VertexOut main_vs(uint vertexId: SV_VertexID) {
    VertexIn vertex_in = static_buffer.Load<VertexIn>(push_constant.vertex_offset + vertexId * 56);
    VertexOut vertex_out;

    float4 pos_view_relative = mul(const_data.view, mul(const_data.world, vertex_in.pos));
    vertex_out.hpos = mul(const_data.projection, pos_view_relative);
    vertex_out.color = vertex_in.color;
    vertex_out.uv_coord = vertex_in.uv_coord;

    return vertex_out;
}

float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    return saturate(vertex_out.color + const_data.color);
}
