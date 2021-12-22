// This is generated file. Do not edit manually

use lgn_graphics_api::DeviceContext;
pub mod cgen_type;
pub mod descriptor_set;
pub mod pipeline_layout;

pub struct CGenDef {
}

pub fn initialize(device_context: &DeviceContext) {
	
	descriptor_set::ViewDescriptorSet::initialize(device_context);
	descriptor_set::FrameDescriptorSet::initialize(device_context);
	
	let descriptor_set_layouts = [
		descriptor_set::ViewDescriptorSet::descriptor_set_layout(),
		descriptor_set::FrameDescriptorSet::descriptor_set_layout(),
	];
	
	pipeline_layout::TmpPipelineLayout::initialize(device_context, &descriptor_set_layouts);
	
}

