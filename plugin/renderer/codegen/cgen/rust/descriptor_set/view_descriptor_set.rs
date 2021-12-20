// This is generated file. Do not edit manually

use lgn_graphics_api::DeviceContext;
use lgn_graphics_api::DescriptorSetLayoutDef;
use lgn_graphics_api::DescriptorSetLayout;
#[allow(unused_imports)]
use super::super::cgen_type::view_data::ViewData;

pub struct ViewDescriptorSet {
	api_layout : DescriptorSetLayout,
}

impl ViewDescriptorSet {
	pub fn new(device_context: &DeviceContext) -> Self {
		let mut layout_def = DescriptorSetLayoutDef::default();
		layout_def.frequency = 0;
		let api_layout = device_context.create_descriptorset_layout(&layout_def).unwrap();
		Self { api_layout }
	}
	pub fn api_layout(&self) -> &DescriptorSetLayout { &self.api_layout }
}
