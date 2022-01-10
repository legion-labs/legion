// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::{
	CGenTypeDef,
};

use lgn_graphics_cgen_runtime::prelude::*;

/*
StructLayout {
    size: 24,
    padded_size: 24,
    members: [
        StructMemberLayout {
            offset: 0,
            absolute_offset: 0,
            size: 12,
            padded_size: 12,
        },
        StructMemberLayout {
            offset: 16,
            absolute_offset: 0,
            size: 8,
            padded_size: 8,
        },
    ],
}
*/
static TYPE_DEF: CGenTypeDef = CGenTypeDef{ 
	name: "LayoutSB",
	id: 8,
	size: 24,
}; 

static_assertions::const_assert_eq!(mem::size_of::<LayoutSB>(), 24);

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct LayoutSB {
	pub a: [Float1; 3],
	pad_0: [u8;4],
	pub b: Float2,
}

impl LayoutSB {
	pub const fn id() -> u32 { 8  }
	
	pub fn def() -> &'static CGenTypeDef { &TYPE_DEF }
	
}

