// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::{
	CGenTypeDef,
};

use lgn_graphics_cgen_runtime::prelude::*;

/*
StructLayout {
    size: 128,
    padded_size: 128,
    members: [
        StructMemberLayout {
            offset: 0,
            absolute_offset: 0,
            size: 64,
            padded_size: 64,
        },
        StructMemberLayout {
            offset: 64,
            absolute_offset: 64,
            size: 64,
            padded_size: 64,
        },
    ],
}
*/
static TYPE_DEF: CGenTypeDef = CGenTypeDef{ 
	name: "ViewData",
	id: 10,
	size: 128,
}; 

static_assertions::const_assert_eq!(mem::size_of::<ViewData>(), 128);

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct ViewData {
	pub view: Float4x4,
	pub projection: Float4x4,
}

impl ViewData {
	pub const fn id() -> u32 { 10  }
	
	pub fn def() -> &'static CGenTypeDef { &TYPE_DEF }
	
}

