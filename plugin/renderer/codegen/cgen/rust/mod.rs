// This is generated file. Do not edit manually

use lgn_graphics_api::DeviceContext;
pub(crate) mod cgen_type;
#[allow(unused_imports)]
pub(crate) use cgen_type::*;
pub(crate) mod descriptor_set;
#[allow(unused_imports)]
pub(crate) use descriptor_set::*;
pub(crate) mod pipeline_layout;
#[allow(unused_imports)]
pub(crate) use pipeline_layout::*;

pub struct CodeGen {
	view_descriptor_set: ViewDescriptorSet,
	frame_descriptor_set: FrameDescriptorSet,
}

impl CodeGen {
	pub fn new(device_context: &DeviceContext) -> Self {
		Self{
			view_descriptor_set: ViewDescriptorSet::new(device_context), 
			frame_descriptor_set: FrameDescriptorSet::new(device_context), 
		}
	}
	pub fn view_descriptor_set(&self) -> &ViewDescriptorSet { &self.view_descriptor_set  }
	pub fn frame_descriptor_set(&self) -> &FrameDescriptorSet { &self.frame_descriptor_set  }
}

