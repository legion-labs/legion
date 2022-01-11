// This is generated file. Do not edit manually

#ifndef DESCRIPTORSET_FRAMEDESCRIPTORSET
#define DESCRIPTORSET_FRAMEDESCRIPTORSET

    #include "../cgen_type/directional_light.hlsl"
    #include "../cgen_type/layout_cb.hlsl"
    #include "../cgen_type/layout_sb.hlsl"
    #include "../cgen_type/layout_sb2.hlsl"
    #include "../cgen_type/omnidirectional_light.hlsl"
    #include "../cgen_type/spotlight.hlsl"

    [[vk::binding(0, 1)]]
    SamplerState  smp;
    [[vk::binding(1, 1)]]
    SamplerState  smp_arr[10];
    [[vk::binding(2, 1)]]
    ConstantBuffer<LayoutCB> cb;
    [[vk::binding(3, 1)]]
    ConstantBuffer<LayoutCB> cb_tr;
    [[vk::binding(4, 1)]]
    StructuredBuffer<LayoutSB> sb;
    [[vk::binding(5, 1)]]
    StructuredBuffer<LayoutSB2> sb2;
    [[vk::binding(6, 1)]]
    StructuredBuffer<LayoutSB> sb_arr[10];
    [[vk::binding(7, 1)]]
    RWStructuredBuffer<LayoutSB> rw_sb;
    [[vk::binding(8, 1)]]
    ByteAddressBuffer bab;
    [[vk::binding(9, 1)]]
    RWByteAddressBuffer rw_bab;
    [[vk::binding(10, 1)]]
    Texture2D<float4> tex2d;
    [[vk::binding(11, 1)]]
    RWTexture2D<float4> rw_tex2d;
    [[vk::binding(12, 1)]]
    Texture3D<float4> tex3d;
    [[vk::binding(13, 1)]]
    RWTexture3D<float4> rw_tex3d;
    [[vk::binding(14, 1)]]
    Texture2DArray<float4> tex2darr;
    [[vk::binding(15, 1)]]
    RWTexture2DArray<float4> rw_tex2darr;
    [[vk::binding(16, 1)]]
    TextureCube<float4> rw_texcube;
    [[vk::binding(17, 1)]]
    TextureCubeArray<float4> rw_texcubearr;
    [[vk::binding(18, 1)]]
    StructuredBuffer<OmnidirectionalLight> sb_omni_lights;
    [[vk::binding(19, 1)]]
    StructuredBuffer<DirectionalLight> sb_dir_lights;
    [[vk::binding(20, 1)]]
    StructuredBuffer<Spotlight> sb_spotlights;

#endif
