// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::{
	CGenTypeDef,
};

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef{ 
	name: "Spotlight",
	id: 13,
	size: mem::size_of::<Spotlight>(),
}; 

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct Spotlight {
	pub pos: Float3,
	pub radiance: Float1,
	pub dir: Float3,
	pub cone_angle: Float1,
	pub color: Float3,
}

impl Spotlight {
	pub const fn id() -> u32 { 13  }
	
	pub fn def() -> &'static CGenTypeDef { &TYPE_DEF }
	
}

