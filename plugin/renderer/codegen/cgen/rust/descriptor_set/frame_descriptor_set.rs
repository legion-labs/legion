// This is generated file. Do not edit manually

use lgn_graphics_cgen_runtime::CGenDescriptorSetId;
use lgn_graphics_cgen_runtime::CGenDescriptorSetInfo;
use lgn_graphics_cgen_runtime::CGenDescriptorId;

static ID : CGenDescriptorSetId = CGenDescriptorSetId(1); 

pub struct FrameDescriptorSet;

#[allow(non_upper_case_globals)]
impl FrameDescriptorSet {
	pub const smp : CGenDescriptorId = CGenDescriptorId(0);
	pub const smp_arr : CGenDescriptorId = CGenDescriptorId(1);
	pub const cb : CGenDescriptorId = CGenDescriptorId(2);
	pub const cb_tr : CGenDescriptorId = CGenDescriptorId(3);
	pub const sb : CGenDescriptorId = CGenDescriptorId(4);
	pub const sb_arr : CGenDescriptorId = CGenDescriptorId(5);
	pub const rw_sb : CGenDescriptorId = CGenDescriptorId(6);
	pub const bab : CGenDescriptorId = CGenDescriptorId(7);
	pub const rw_bab : CGenDescriptorId = CGenDescriptorId(8);
	pub const tex2d : CGenDescriptorId = CGenDescriptorId(9);
	pub const rw_tex2d : CGenDescriptorId = CGenDescriptorId(10);
	pub const tex3d : CGenDescriptorId = CGenDescriptorId(11);
	pub const rw_tex3d : CGenDescriptorId = CGenDescriptorId(12);
	pub const tex2darr : CGenDescriptorId = CGenDescriptorId(13);
	pub const rw_tex2darr : CGenDescriptorId = CGenDescriptorId(14);
	pub const rw_texcube : CGenDescriptorId = CGenDescriptorId(15);
	pub const rw_texcubearr : CGenDescriptorId = CGenDescriptorId(16);
	
	pub fn id() -> CGenDescriptorSetId { ID }
} // FrameDescriptorSet

impl CGenDescriptorSetInfo for FrameDescriptorSet {
	fn id() -> CGenDescriptorSetId { Self::id() }
} // CGenDescriptorSetInfo

