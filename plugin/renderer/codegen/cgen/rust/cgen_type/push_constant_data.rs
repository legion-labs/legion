// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::{
	CGenTypeDef,
};

use lgn_graphics_cgen_runtime::prelude::*;

/*
StructLayout {
    size: 16,
    padded_size: 16,
    members: [
        StructMemberLayout {
            offset: 0,
            absolute_offset: 0,
            size: 16,
            padded_size: 16,
        },
    ],
}
*/
static TYPE_DEF: CGenTypeDef = CGenTypeDef{ 
	name: "PushConstantData",
	id: 11,
	size: 16,
}; 

static_assertions::const_assert_eq!(mem::size_of::<PushConstantData>(), 16);

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct PushConstantData {
	pub color: Float4,
}

impl PushConstantData {
	pub const fn id() -> u32 { 11  }
	
	pub fn def() -> &'static CGenTypeDef { &TYPE_DEF }
	
}

