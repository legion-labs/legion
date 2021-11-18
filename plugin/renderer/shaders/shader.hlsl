struct VertexIn {
    float3 pos : POSITION;    
};

struct VertexOut {  
    float4 hpos : SV_POSITION;
};

struct PushConstData {
    float4x4 world;
    float4x4 view;
    float4x4 projection;
    float4 color;
};

struct ConstData {
    float4 uniform_color;
};

ConstantBuffer<ConstData> uniform_data;

[[vk::push_constant]]
ConstantBuffer<PushConstData> push_constant;


VertexOut main_vs(in VertexIn vertex_in) {
    VertexOut vertex_out;
    vertex_out.hpos = mul(push_constant.projection, mul(push_constant.view, mul(push_constant.world, float4(vertex_in.pos, 1.0))));
    return vertex_out;
}



float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    float4 color = uniform_data.uniform_color;
    return push_constant.color;
}