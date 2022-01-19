// This is generated file. Do not edit manually

#ifndef PIPELINELAYOUT_SHADERPIPELINELAYOUT
#define PIPELINELAYOUT_SHADERPIPELINELAYOUT

    // DescriptorSets
    // - name: frame_descriptor_set
    // - freq: 0
    #include "../descriptor_set/frame_descriptor_set.hlsl"

    // - name: view_descriptor_set
    // - freq: 1
    #include "../descriptor_set/view_descriptor_set.hlsl"

    // PushConstant
    // - name: push_constant
    #include "../cgen_type/instance_push_constant_data.hlsl"

    [[vk::push_constant]]
    ConstantBuffer<InstancePushConstantData> push_constant; 


#endif
