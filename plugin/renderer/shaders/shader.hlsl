struct VertexIn {
    float4 pos : POSITION;    
};

struct VertexOut {  
    float4 hpos : SV_POSITION;
};

VertexOut main_vs(in VertexIn vertex_in) {
    VertexOut vertex_out;
    vertex_out.hpos = vertex_in.pos;    
    return vertex_out;
}

struct ConstData {
    float4 uniform_color;
};

ConstantBuffer<ConstData> uniform_data;

float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    return uniform_data.uniform_color;
}