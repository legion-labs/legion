struct VertexIn {
    float3 pos : POSITION;
};

struct VertexOut {
    float4 hpos : SV_POSITION;
    float4 color : COLOR;
};

struct VertexColor {
    float3 foo;    
    float4 color;
    float bar;
};

[[vk::binding(0, 0)]]
RWByteAddressBuffer rw_test_byteaddressbuffer[1];

[[vk::binding(1, 0)]]
ByteAddressBuffer test_byteaddressbuffer[2];

[[vk::binding(2, 0)]]
StructuredBuffer<VertexColor> vertex_color;

[[vk::binding(3, 0)]]
ConstantBuffer<VertexColor> cb_vertex_color[6];

[[vk::binding(4, 0)]]
SamplerState sampl[12];

[[vk::binding(5, 0)]]
Texture2D<float3> tex2d[2];

[[vk::binding(6, 0)]]
RWTexture2D<float3> rw_tex2d[2];

[[vk::binding(7, 0)]]
Texture2DArray<float3> tex2darray[2];

[[vk::binding(8, 0)]]
RWTexture2DArray<float3> rw_tex2darray[2];

[[vk::binding(9, 0)]]
Texture3D<float2> tex3d[2];

[[vk::binding(10, 0)]]
RWTexture3D<float2> rw_tex3d[2];

[[vk::binding(11, 0)]]
TextureCube<float> texcube[2];

[[vk::binding(12, 0)]]
TextureCubeArray<float> rw_texcube[2];

[[vk::binding(13, 0)]]
RWStructuredBuffer<VertexColor> rw_vertex_color;

[[vk::push_constant]]
ConstantBuffer<VertexColor> push_cst;

VertexOut main_vs(in VertexIn vIn) {

    VertexOut vOut;
    vOut.hpos = float4(vIn.pos, 1.f);
    vOut.color = vertex_color[0].color;   
    vOut.color = push_cst.color;
    return vOut;
}

float4 main_ps(in VertexOut fIn) : SV_TARGET0  {
    return fIn.color;
}