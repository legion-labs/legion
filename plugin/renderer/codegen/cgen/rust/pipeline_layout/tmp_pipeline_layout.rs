// This is generated file. Do not edit manually

use lgn_graphics_api::DeviceContext;
use lgn_graphics_api::RootSignature;
use lgn_graphics_api::DescriptorSetLayout;
use lgn_graphics_api::DescriptorSetHandle;
use lgn_graphics_api::MAX_DESCRIPTOR_SET_LAYOUTS;
use lgn_graphics_cgen_runtime::CGenPipelineLayoutDef;

use super::super::descriptor_set::ViewDescriptorSet;

static pipeline_layout_def: CGenPipelineLayoutDef = CGenPipelineLayoutDef{ 
	name: "TmpPipelineLayout",
	id: 0,
	descriptor_set_layout_ids: [
	Some(ViewDescriptorSet::id()),
	None,
	None,
	None,
	],
	push_constant_type: None,
}; 

static mut pipeline_layout: Option<RootSignature> = None;

pub struct TmpPipelineLayout {
	descriptor_sets: [Option<DescriptorSetHandle>; MAX_DESCRIPTOR_SET_LAYOUTS],
}

impl TmpPipelineLayout {
	
	#![allow(unsafe_code)]
	pub fn initialize(device_context: &DeviceContext, descriptor_set_layouts: &[&DescriptorSetLayout]) {
		unsafe { pipeline_layout = Some(pipeline_layout_def.create_pipeline_layout(device_context, descriptor_set_layouts)); }
	}
	
	pub fn set_view_descriptor_set(&mut self, descriptor_set_handle: DescriptorSetHandle) {
	}
}

