// This is generated file. Do not edit manually

use lgn_graphics_cgen_runtime::CGenPipelineLayoutId;
use lgn_graphics_cgen_runtime::CGenPipelineLayoutInfo;

static ID : CGenPipelineLayoutId = CGenPipelineLayoutId(0); 

pub struct TmpPipelineLayout;

impl TmpPipelineLayout {
	pub fn id() -> CGenPipelineLayoutId { ID }
} // TmpPipelineLayout

impl CGenPipelineLayoutInfo for TmpPipelineLayout {
	fn id() -> CGenPipelineLayoutId { ID }
} // CGenPipelineLayoutInfo

