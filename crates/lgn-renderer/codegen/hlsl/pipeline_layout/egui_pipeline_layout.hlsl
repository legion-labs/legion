// This is generated file. Do not edit manually

#ifndef PIPELINE_LAYOUT_EGUIPIPELINELAYOUT
#define PIPELINE_LAYOUT_EGUIPIPELINELAYOUT

    // DescriptorSets
    // - name: descriptor_set
    // - freq: 0
    #include "../descriptor_set/egui_descriptor_set.hlsl"

    // PushConstant
    // - name: push_constant
    #include "../cgen_type/egui_push_constant_data.hlsl"

    [[vk::push_constant]]
    ConstantBuffer<EguiPushConstantData> push_constant; 

#endif
