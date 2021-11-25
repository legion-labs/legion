struct VertexIn {
    float2 pos : POSITION;
    float2 uv : TEXCOORD;
    float4 color : COLOR;
};

struct VertexOut {  
    float4 hpos : SV_POSITION;
    float2 uv : TEXCOORD;
    float4 color : COLOR;
};

struct PushConstData {
    float2 scale;
    float2 translation;
};

[[vk::push_constant]]
ConstantBuffer<PushConstData> push_constant;

VertexOut main_vs(in VertexIn vertex_in) {
    VertexOut vertex_out;
    vertex_out.hpos = float4(vertex_in.pos * push_constant.scale + push_constant.translation, 0.0, 1.0);
    vertex_out.uv = vertex_in.uv;
    vertex_out.color = vertex_in.color; //TODO: sRGB
    return vertex_out;
}

[[vk::binding(0)]]
Texture2D font_texture : register(t0);
[[vk::binding(1)]]
SamplerState font_sampler : register(s0);

float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    return font_texture.Sample(font_sampler, vertex_out.uv);
}