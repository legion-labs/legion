struct VertexIn {
    float4 pos : POSITION;    
};

struct VertexOut {  
    float4 hpos : SV_POSITION;
};

struct ConstData {
    float4 uniform_color;
    float4x4 world;
    float4x4 view;
    float4x4 projection;
};

ConstantBuffer<ConstData> uniform_data;

[[vk::push_constant]]
ConstantBuffer<ConstData> push_constant;


VertexOut main_vs(in VertexIn vertex_in) {
    VertexOut vertex_out;
    vertex_out.hpos = mul(push_constant.projection, mul(push_constant.view, mul(push_constant.world, vertex_in.pos)));
    return vertex_out;
}



float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    return uniform_data.uniform_color * push_constant.uniform_color;
}