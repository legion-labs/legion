struct VertexIn {
    float4 pos : POSITION;    
};

struct VertexOut {  
    float4 hpos : SV_POSITION;
};

struct ConstData {
    float4 uniform_color;
    float4 translation;
    float4 scale;
};

ConstantBuffer<ConstData> uniform_data;

[[vk::push_constant]]
ConstantBuffer<ConstData> push_constant;


VertexOut main_vs(in VertexIn vertex_in) {
    VertexOut vertex_out;
    vertex_out.hpos = vertex_in.pos * push_constant.scale + push_constant.translation;
    return vertex_out;
}



float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    return uniform_data.uniform_color * push_constant.uniform_color;
}