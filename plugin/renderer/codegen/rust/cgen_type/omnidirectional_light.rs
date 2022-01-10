// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::{
	CGenTypeDef,
};

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef{ 
	name: "OmnidirectionalLight",
	id: 11,
	size: mem::size_of::<OmnidirectionalLight>(),
}; 

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct OmnidirectionalLight {
	pub pos: Float3,
	pub radiance: Float1,
	pub color: Float3,
}

impl OmnidirectionalLight {
	pub const fn id() -> u32 { 11  }
	
	pub fn def() -> &'static CGenTypeDef { &TYPE_DEF }
	
}

