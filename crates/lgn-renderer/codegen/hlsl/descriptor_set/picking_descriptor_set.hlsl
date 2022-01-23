// This is generated file. Do not edit manually

#ifndef DESCRIPTOR_SET_PICKINGDESCRIPTORSET
#define DESCRIPTOR_SET_PICKINGDESCRIPTORSET

    #include "crate://renderer/codegen/hlsl/cgen_type/picking_data.hlsl"

    [[vk::binding(0, 2)]]
    RWStructuredBuffer<uint> picked_count;

    [[vk::binding(1, 2)]]
    RWStructuredBuffer<PickingData> picked_objects;

#endif
