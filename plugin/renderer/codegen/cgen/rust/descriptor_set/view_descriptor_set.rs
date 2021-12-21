// This is generated file. Do not edit manually

use lgn_graphics_cgen_runtime::CGenDescriptorSetId;
use lgn_graphics_cgen_runtime::CGenDescriptorSetInfo;
use lgn_graphics_cgen_runtime::CGenDescriptorId;

static ID : CGenDescriptorSetId = CGenDescriptorSetId(0); 

pub struct ViewDescriptorSet;

#[allow(non_upper_case_globals)]
impl ViewDescriptorSet {
	pub const view_data : CGenDescriptorId = CGenDescriptorId(0);
	
	pub fn id() -> CGenDescriptorSetId { ID }
} // ViewDescriptorSet

impl CGenDescriptorSetInfo for ViewDescriptorSet {
	fn id() -> CGenDescriptorSetId { Self::id() }
} // CGenDescriptorSetInfo

