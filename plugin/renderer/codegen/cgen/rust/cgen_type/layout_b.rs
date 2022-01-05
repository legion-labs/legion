// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::{
	CGenTypeDef,
};

use super::layout_a::LayoutA;
use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef{ 
	name: "LayoutB",
	id: 8,
	size: mem::size_of::<LayoutB>(),
}; 

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct LayoutB {
	pub a: Float3,
	pub b: Float4,
	pub c: LayoutA,
}

impl LayoutB {
	pub const fn id() -> u32 { 8  }
	
	pub fn def() -> &'static CGenTypeDef { &TYPE_DEF }
	
}

