struct VertexIn {
    float3 pos : POSITION;
    float3 normal : NORMAL;
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
    float circle_half_width;
};

ConstantBuffer<ConstData> const_data;

VertexOut main_vs(in VertexIn vertex_in) {
    VertexOut vertex_out;

    float4 pos_view_relative = mul(const_data.view, mul(const_data.world, float4(vertex_in.pos, 1.0)));
    vertex_out.hpos = mul(const_data.projection, pos_view_relative);
    vertex_out.color = vertex_in.color;
    vertex_out.uv_coord = vertex_in.uv_coord;

    return vertex_out;
}

float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    // if (const_data.circle_half_width > 0)
    // {
    //     if (abs(dot(vertex_out.uv_coord, vertex_out.uv_coord) - 1.0f + const_data.circle_half_width) < const_data.circle_half_width)
    //         clip(-1);
    // }

    return vertex_out.color + const_data.color;
}
