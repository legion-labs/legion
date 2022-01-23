// This is generated file. Do not edit manually

#ifndef PIPELINE_LAYOUT_PICKINGPIPELINELAYOUT
#define PIPELINE_LAYOUT_PICKINGPIPELINELAYOUT

    // DescriptorSets
    // - name: frame_descriptor_set
    // - freq: 0
    #include "crate://renderer/codegen/hlsl/descriptor_set/frame_descriptor_set.hlsl"

    // - name: view_descriptor_set
    // - freq: 1
    #include "crate://renderer/codegen/hlsl/descriptor_set/view_descriptor_set.hlsl"

    // - name: picking_descriptor_set
    // - freq: 2
    #include "crate://renderer/codegen/hlsl/descriptor_set/picking_descriptor_set.hlsl"

    // PushConstant
    // - name: push_constant
    #include "crate://renderer/codegen/hlsl/cgen_type/picking_push_constant_data.hlsl"

    [[vk::push_constant]]
    ConstantBuffer<PickingPushConstantData> push_constant; 

#endif
