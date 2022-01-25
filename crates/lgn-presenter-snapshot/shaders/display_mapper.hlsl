struct VertexOut {
    float4 pos: SV_POSITION;
    float2 uv: TEXCOORD0;
};

VertexOut main_vs(uint vertex_id: SV_VERTEXID) {
    VertexOut result;

    if( vertex_id == 0) {
        result.pos = float4(-1.f, 1.f, 0.f, 1.f);
        result.uv = float2(0.f, 0.f);
    } else if( vertex_id == 1) {
        result.pos = float4(-1.f, -3.f, 0.f, 1.f);
        result.uv = float2(0.f, 2.f);
    } else if( vertex_id == 2) {
        result.pos = float4(3.f, 1.f, 0.f, 1.f);
        result.uv = float2(2.f, 0.f);
    }

    return result;
}

float4 main_ps(in VertexOut vertex_out) : SV_TARGET {

    float4 value = hdr_image.Sample(hdr_sampler, vertex_out.uv );

    return value;
}