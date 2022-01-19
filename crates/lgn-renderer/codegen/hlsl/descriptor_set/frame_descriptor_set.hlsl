// This is generated file. Do not edit manually

#ifndef DESCRIPTORSET_FRAMEDESCRIPTORSET
#define DESCRIPTORSET_FRAMEDESCRIPTORSET

    #include "../cgen_type/directional_light.hlsl"
    #include "../cgen_type/lighting_data.hlsl"
    #include "../cgen_type/omni_directional_light.hlsl"
    #include "../cgen_type/spot_light.hlsl"

    [[vk::binding(0, 0)]]
    ConstantBuffer<LightingData> lighting_data;
    [[vk::binding(1, 0)]]
    StructuredBuffer<DirectionalLight> directional_lights;
    [[vk::binding(2, 0)]]
    StructuredBuffer<OmniDirectionalLight> omni_directional_lights;
    [[vk::binding(3, 0)]]
    StructuredBuffer<SpotLight> spot_lights;
    [[vk::binding(4, 0)]]
    ByteAddressBuffer static_buffer;

#endif
