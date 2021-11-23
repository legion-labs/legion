use lgn_graphics_api::DeviceContext;
pub(crate) mod c_gen_type;
pub(crate) use c_gen_type::*;
pub(crate) mod descriptor_set;
pub(crate) use descriptor_set::*;
pub(crate) mod pipeline_layout;
pub(crate) use pipeline_layout::*;

pub struct CodeGen {
	frame_descriptor_set: FrameDescriptorSet,
}

impl CodeGen {
	pub fn new(device_context: &DeviceContext) -> Self {
		Self{
			frame_descriptor_set: FrameDescriptorSet::new(device_context), 
		}
	}
	pub fn frame_descriptor_set(&self) -> &FrameDescriptorSet { &self.frame_descriptor_set  }
}

