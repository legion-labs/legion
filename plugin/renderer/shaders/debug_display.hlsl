struct VertexIn {
    float3 pos : POSITION;
};

struct VertexOut {  
    float4 hpos : SV_POSITION;
    float3 pos : POSITION;
};

struct ConstData {
    float4x4 world;
    float4x4 view;
    float4x4 projection;
    float4 color;
};

struct PushConstData {
    uint offset;
};

struct EntityTransforms {
    float4x4 world;
};

ConstantBuffer<ConstData> const_data;

VertexOut main_vs(in VertexIn vertex_in) {
    VertexOut vertex_out;

    float4x4 world = transpose(const_data.world);

    float4 pos_view_relative = mul(const_data.view, mul(world, float4(vertex_in.pos, 1.0)));
    vertex_out.hpos = mul(const_data.projection, pos_view_relative);
    vertex_out.pos = pos_view_relative.xyz;
    return vertex_out;
}

#define PI 3.141592

float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    return const_data.color;
}