// This is generated file. Do not edit manually

#ifndef DESCRIPTORSET_FRAMEDESCRIPTORSET
#define DESCRIPTORSET_FRAMEDESCRIPTORSET

	#include "../c_gen_type/layout_a.hlsl"
	
	[[vk::binding(0, 1)]]
	SamplerState  smp;
	[[vk::binding(1, 1)]]
	SamplerState  smp_arr[10];
	[[vk::binding(2, 1)]]
	ConstantBuffer<LayoutA> cb;
	[[vk::binding(3, 1)]]
	ConstantBuffer<LayoutA> cb_tr;
	[[vk::binding(4, 1)]]
	StructuredBuffer<LayoutA> sb;
	[[vk::binding(5, 1)]]
	StructuredBuffer<LayoutA> sb_arr[10];
	[[vk::binding(6, 1)]]
	RWStructuredBuffer<LayoutA> rw_sb;
	[[vk::binding(7, 1)]]
	ByteAddressBuffer bab;
	[[vk::binding(8, 1)]]
	RWByteAddressBuffer rw_bab;
	[[vk::binding(9, 1)]]
	Texture2D<float4> tex2d;
	[[vk::binding(10, 1)]]
	RWTexture2D<float4> rw_tex2d;
	[[vk::binding(11, 1)]]
	Texture3D<float4> tex3d;
	[[vk::binding(12, 1)]]
	RWTexture3D<float4> rw_tex3d;
	[[vk::binding(13, 1)]]
	Texture2DArray<float4> tex2darr;
	[[vk::binding(14, 1)]]
	RWTexture2DArray<float4> rw_tex2darr;
	[[vk::binding(15, 1)]]
	TextureCube<float4> rw_texcube;
	[[vk::binding(16, 1)]]
	TextureCubeArray<float4> rw_texcubearr;
	
#endif
