// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::{
	CGenTypeDef,
};

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef{ 
	name: "PushConstantData",
	id: 10,
	size: mem::size_of::<PushConstantData>(),
}; 

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct PushConstantData {
	pub color: Float4,
}

impl PushConstantData {
	pub const fn id() -> u32 { 10  }
	
	pub fn def() -> &'static CGenTypeDef { &TYPE_DEF }
	
}

