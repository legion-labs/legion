// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::{
	CGenTypeDef,
};

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef{ 
	name: "DirectionalLight",
	id: 12,
	size: mem::size_of::<DirectionalLight>(),
}; 

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct DirectionalLight {
	pub dir: Float3,
	pub radiance: Float1,
	pub color: Float3,
}

impl DirectionalLight {
	pub const fn id() -> u32 { 12  }
	
	pub fn def() -> &'static CGenTypeDef { &TYPE_DEF }
	
}

