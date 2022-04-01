#include "crate://lgn-streamer/gpu/pipeline_layout/rgb2yuv_pipeline_layout.hlsl"

#include "crate://lgn-graphics-renderer/gpu/include/common.hsh"

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// https://en.wikipedia.org/wiki/YCbCr
// https://docs.microsoft.com/en-us/windows/win32/medfound/recommended-8-bit-yuv-formats-for-video-rendering
// https://www.itu.int/dms_pubrec/itu-r/rec/bt/R-REC-BT.709-6-201506-I!!PDF-E.pdf
// https://www.itu.int/dms_pubrec/itu-r/rec/bt/R-REC-BT.2020-2-201510-I!!PDF-E.pdf
float4x4 rgb2yuv_matrix(float kr, float kb, int black, int white, int mid, int maxi) {
    float kg = 1.0f - kr - kb;
    float c = 1.0f * (white - black) / maxi;
    return float4x4(
        c * kr,                       c * kg,                       c * kb,                       1.0f * black / maxi,
        c * -0.5f * kr / (1.0f - kb), c * -0.5f * kg / (1.0f - kb), c * 0.5f,                     1.0f * mid / maxi,
        c * 0.5f,                     c * -0.5f * kg / (1.0f - kr), c * -0.5f * kb / (1.0f - kr), 1.0f * mid / maxi,
        0,                            0,                            0,                            1.0f
    );
}

static float4x4 rgb2yuv_bt709 = rgb2yuv_matrix(0.2126f, 0.0722f, 16, 235, 128, 255); // 8bit
static float4x4 rgb2yuv_bt2020 = rgb2yuv_matrix(0.2627f, 0.0593f, 64 , 940, 512, 1023); // 10 bits

float3 rgb2yuv(float3 rgb) {
    return mul(rgb2yuv_bt709, float4(rgb, 1)).xyz;
}

#define UV_TILE_SIZE 8

groupshared float2 gs_uv[UV_TILE_SIZE][UV_TILE_SIZE];

/// Convert RGB to YUV (rec.709)
[numthreads(UV_TILE_SIZE, UV_TILE_SIZE, 1)]
void main_cs(uint3 dispatchThreadId : SV_DispatchThreadID,
             uint3 groupThreadId    : SV_GroupThreadID) {

    uint2 screenPos = dispatchThreadId.xy;   
    uint2 tilePos   = groupThreadId.xy; 

    float3 rgb = linear2srgb(tonemap(hdr_image[screenPos].rgb));
    float3 yuv = rgb2yuv(rgb);

    y_image[screenPos] = yuv.x;
    gs_uv[tilePos.x][tilePos.y] = yuv.yz;

    GroupMemoryBarrierWithGroupSync();

    if (((tilePos.x & 1) == 0) && ((tilePos.y & 1) == 0)) {
        float2 pix0x0 = gs_uv[tilePos.x][tilePos.y];
        float2 pix0x1 = gs_uv[tilePos.x][tilePos.y + 1];
        float2 pix1x0 = gs_uv[tilePos.x + 1][tilePos.y];
        float2 pix1x1 = gs_uv[tilePos.x + 1][tilePos.y + 1];
        float2 uv = (pix0x0 + pix0x1 + pix1x0 + pix1x1) * 0.25f;
        u_image[screenPos >> 1] = uv.x;
        v_image[screenPos >> 1] = uv.y;
    }
}
