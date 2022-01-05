// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::{
	CGenTypeDef,
};

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef{ 
	name: "LayoutA",
	id: 7,
	size: mem::size_of::<LayoutA>(),
}; 

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct LayoutA {
	pub a: Float1,
	pub b: Float2,
}

impl LayoutA {
	pub const fn id() -> u32 { 7  }
	
	pub fn def() -> &'static CGenTypeDef { &TYPE_DEF }
	
}

