#ifndef PIPELINELAYOUT_DEFAULTPL
#define PIPELINELAYOUT_DEFAULTPL

	// DescriptorSets
	// - name: frame_set
	// - freq: 1
	#include "../descriptor_set/frame_descriptor_set.hlsl"
	
	// PushConstant
	// - name: b
	#include "../c_gen_type/layout_a.hlsl"
	
	[[vk::push_constant]]
	ConstantBuffer<LayoutA> b; 
	
	
#endif
