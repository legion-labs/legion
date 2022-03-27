#include "crate://lgn-graphics-renderer/gpu/pipeline_layout/final_resolve_pipeline_layout.hlsl"

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// https://en.wikipedia.org/wiki/SRGB
float linear2srgb_std(float value) {
    return (value <= 0.0031308f) ? (12.92f * value) : (1.055f * pow(value, 1.0f / 2.4f) - 0.055f);
}

float3 linear2srgb(float3 value)
{
#if 0
    return pow(value, 1.0f/2.2f);
#else
    return float3(linear2srgb_std(value.r), linear2srgb_std(value.g), linear2srgb_std(value.b));
#endif
}

// https://en.wikipedia.org/wiki/SRGB
// putting the reciprocal here for now until we move color utilities to a header
float srgb2linear_std(float value) {
    return (value <= 0.04045f) ? (value / 12.92f) : pow((value + 0.055f) / 1.055f, 2.4f);
}

float3 srgb2linear(float3 value)
{
#if 0
    return pow(value, 2.2f);
#else
    return float3(srgb2linear_std(value.r), srgb2linear_std(value.g), srgb2linear_std(value.b));
#endif
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

float3 tonemap(float3 value) {
    // place holder
    return saturate(value);
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

struct VertexIn {
    float2 pos : POSITION;
    float2 uv : TEXCOORD;
};

struct VertexOut {  
    float4 hpos : SV_POSITION;
    float2 uv : TEXCOORD;
};

VertexOut main_vs(in VertexIn vertex_in) {
    VertexOut vertex_out;

    vertex_out.hpos = float4(float2(2.0 * vertex_in.pos.x - 1.0, 1.0 - 2.0 * vertex_in.pos.y), 0.0, 1.0);
    vertex_out.uv = vertex_in.uv;

    return vertex_out;
}

float4 main_ps(in VertexOut vertex_out) : SV_TARGET {
    float4 hdr_image = linear_texture.Sample(linear_sampler, vertex_out.uv);
    return float4(linear2srgb(tonemap(hdr_image.rgb)), 1.0);
}
