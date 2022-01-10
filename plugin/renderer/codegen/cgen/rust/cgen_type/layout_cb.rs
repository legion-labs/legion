// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::{
	CGenTypeDef,
};

use lgn_graphics_cgen_runtime::prelude::*;

/*
StructLayout {
    size: 64,
    padded_size: 64,
    members: [
        StructMemberLayout {
            offset: 0,
            absolute_offset: 0,
            size: 48,
            padded_size: 48,
        },
        StructMemberLayout {
            offset: 48,
            absolute_offset: 48,
            size: 8,
            padded_size: 16,
        },
    ],
}
*/
static TYPE_DEF: CGenTypeDef = CGenTypeDef{ 
	name: "LayoutCB",
	id: 7,
	size: 64,
}; 

static_assertions::const_assert_eq!(mem::size_of::<LayoutCB>(), 64);

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct LayoutCB {
	pub a: [Float1; 3],
	pub b: Float2,
}

impl LayoutCB {
	pub const fn id() -> u32 { 7  }
	
	pub fn def() -> &'static CGenTypeDef { &TYPE_DEF }
	
}

