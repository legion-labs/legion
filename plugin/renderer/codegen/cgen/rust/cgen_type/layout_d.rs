// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::{
	CGenTypeDef,
};

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef{ 
	name: "LayoutD",
	id: 5,
	size: mem::size_of::<LayoutD>(),
}; 

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct LayoutD {
	pub a: Float4,
}

impl LayoutD {
	pub const fn id() -> u32 { 5  }
	
	pub fn def() -> &'static CGenTypeDef { &TYPE_DEF }
	
}

