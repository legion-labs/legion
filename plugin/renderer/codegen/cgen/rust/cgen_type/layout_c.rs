// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::{
	CGenTypeDef,
};

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef{ 
	name: "LayoutC",
	id: 6,
	size: mem::size_of::<LayoutC>(),
}; 

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct LayoutC {
	pub a: Float1,
}

impl LayoutC {
	pub const fn id() -> u32 { 6  }
	
	pub fn def() -> &'static CGenTypeDef { &TYPE_DEF }
	
}

