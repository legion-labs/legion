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
            array_stride: 0,
        },
    ],
}
*/
static TYPE_DEF: CGenTypeDef = CGenTypeDef{ 
	name: "PushConstantData",
	id: 19,
	size: 16,
}; 

static_assertions::const_assert_eq!(mem::size_of::<PushConstantData>(), 16);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct PushConstantData {
	data: [u8;16]
}

impl PushConstantData {
	pub const fn id() -> u32 { 19  }
	
	pub fn def() -> &'static CGenTypeDef { &TYPE_DEF }
	
	pub fn set_color(&mut self, value: Float4) { 
		self.set(0, value);
	}
	
	pub fn color(&self) -> Float4 { 
		self.get(0)
	}
	
	#[allow(unsafe_code)]
	fn set<T: Copy>(&mut self, offset: usize, value: T) {
		unsafe{
			let p = self.data.as_mut_ptr();
			let p = p.add(offset as usize);
			let p = p as *mut T;
			p.write(value);
		}
	}
	
	#[allow(unsafe_code)]
	fn get<T: Copy>(&self, offset: usize) -> T {
		unsafe{
			let p = self.data.as_ptr();
			let p = p.add(offset as usize);
			let p = p as *const T;
			*p
		}
	}
}

impl Default for PushConstantData {
	fn default() -> Self {
		let mut ret = Self {
		data: [0;16]
		};
		ret.set_color(Float4::default());
		ret
	}
}

