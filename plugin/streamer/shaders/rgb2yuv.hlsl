Texture2D<float4> hdr_image;
RWTexture2D<float> y_image;
RWTexture2D<float> u_image;
RWTexture2D<float> v_image;

#define UV_TILE_SIZE 8

groupshared float2 gs_uv[UV_TILE_SIZE][UV_TILE_SIZE];

/// Convert RGB to YUV (rec.709)
[numthreads(UV_TILE_SIZE, UV_TILE_SIZE, 1)]
void cs_main(uint3 dispatchThreadId : SV_DispatchThreadID,
             uint3 groupThreadId    : SV_GroupThreadID) {

    uint2 screenPos = dispatchThreadId.xy;   
    uint2 tilePos   = groupThreadId.xy; 
                 
    float4 rgb = saturate(hdr_image[screenPos]) * 255.0f;

    float y = 0.2578125f * rgb.r + 0.50390625f * rgb.g + 0.09765625f * rgb.b + 16.0f;
    float u = -0.1484375f * rgb.r -0.2890625f * rgb.g + 0.4375f * rgb.b + 128.0f;
    float v = 0.4375f * rgb.r -0.3671875f * rgb.g -0.0703125f * rgb.b + 128.0f;

    y_image[screenPos] = y / 255.0f;
    
    gs_uv[tilePos.x][tilePos.y] = float2(u, v) / 255.0f;

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
