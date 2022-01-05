// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::{
	CGenTypeDef,
};

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef{ 
	name: "ViewData",
	id: 9,
	size: mem::size_of::<ViewData>(),
}; 

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct ViewData {
	pub view: Float4x4,
	pub projection: Float4x4,
}

impl ViewData {
	pub const fn id() -> u32 { 9  }
	
	pub fn def() -> &'static CGenTypeDef { &TYPE_DEF }
	
}

