struct VertexIn {
    float3 pos : POSITION;
};

struct VertexOut {
    float4 hpos : SV_POSITION;
    float4 color : COLOR;
};

struct VertexColor {
    float4 color;
};

// [[vk::binding(0, 0)]]
// StructuredBuffer<VertexColor> sb_vertex_color;

// [[vk::binding(2, 0)]]
// RWStructuredBuffer<VertexColor> rw_vertex_color_fake;

[[vk::binding(1, 0)]]
ConstantBuffer<VertexColor> cb_vertex_color;

// [[vk::binding(0, 1)]]
// ByteAddressBuffer test_byteaddressbuffer;

// [[vk::binding(1, 1)]]
// RWByteAddressBuffer rw_test_byteaddressbuffer;

// [[vk::binding(0, 2)]]
// RWTexture3D<float4> rw_tex3d;

// [numthreads(8,8,8)]
// void main_cs(uint3 gid: SV_DispatchThreadID) {
//     rw_tex3d[gid] = (float4)0.25;
// }

VertexOut main_vs(in VertexIn vIn) {

    VertexOut vOut;
    vOut.hpos = float4(vIn.pos, 1.f);
    vOut.color = cb_vertex_color.color;    
    return vOut;
}

float4 main_ps(in VertexOut fIn) : SV_TARGET0  {
    return fIn.color;
}