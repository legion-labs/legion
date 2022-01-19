// This is generated file. Do not edit manually

#ifndef DESCRIPTORSET_PICKINGDESCRIPTORSET
#define DESCRIPTORSET_PICKINGDESCRIPTORSET

    #include "../cgen_type/picking_data.hlsl"

    [[vk::binding(0, 2)]]
    RWStructuredBuffer<uint> picked_count;
    [[vk::binding(1, 2)]]
    RWStructuredBuffer<PickingData> picked_objects;

#endif
