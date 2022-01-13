// This is generated file. Do not edit manually

#ifndef DESCRIPTORSET_FRAMEDESCRIPTORSET
#define DESCRIPTORSET_FRAMEDESCRIPTORSET

    #include "../cgen_type/directional_light.hlsl"
    #include "../cgen_type/omnidirectional_light.hlsl"
    #include "../cgen_type/spotlight.hlsl"

    [[vk::binding(0, 0)]]
    StructuredBuffer<OmnidirectionalLight> sb_omni_lights;
    [[vk::binding(1, 0)]]
    StructuredBuffer<DirectionalLight> sb_dir_lights;
    [[vk::binding(2, 0)]]
    StructuredBuffer<Spotlight> sb_spotlights;

#endif
