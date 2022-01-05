// This is generated file. Do not edit manually

#ifndef PIPELINELAYOUT_TMPPIPELINELAYOUT
#define PIPELINELAYOUT_TMPPIPELINELAYOUT

	// DescriptorSets
	// - name: view_descriptor_set
	// - freq: 0
	#include "../descriptor_set/view_descriptor_set.hlsl"
	
	// PushConstant
	// - name: push_constant
	#include "../cgen_type/push_constant_data.hlsl"
	
	[[vk::push_constant]]
	ConstantBuffer<PushConstantData> push_constant; 
	
	
#endif
