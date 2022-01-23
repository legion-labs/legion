// This is generated file. Do not edit manually

#ifndef PIPELINE_LAYOUT_CONSTCOLORPIPELINELAYOUT
#define PIPELINE_LAYOUT_CONSTCOLORPIPELINELAYOUT

    // DescriptorSets
    // - name: frame_descriptor_set
    // - freq: 0
    #include "crate://lgn-renderer/codegen/hlsl/descriptor_set/frame_descriptor_set.hlsl"

    // - name: view_descriptor_set
    // - freq: 1
    #include "crate://lgn-renderer/codegen/hlsl/descriptor_set/view_descriptor_set.hlsl"

    // PushConstant
    // - name: push_constant
    #include "crate://lgn-renderer/codegen/hlsl/cgen_type/const_color_push_constant_data.hlsl"

    [[vk::push_constant]]
    ConstantBuffer<ConstColorPushConstantData> push_constant; 

#endif
