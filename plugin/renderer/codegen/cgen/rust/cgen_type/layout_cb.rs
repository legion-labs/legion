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
            array_stride: 16,
        },
        StructMemberLayout {
            offset: 48,
            absolute_offset: 48,
            size: 8,
            padded_size: 16,
            array_stride: 0,
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

#[derive(Clone, Copy)]
#[repr(C)]
pub struct LayoutCB {
	data: [u8;64]
}

impl LayoutCB {
	pub const fn id() -> u32 { 7  }
	
	pub fn def() -> &'static CGenTypeDef { &TYPE_DEF }
	
	pub fn set_a(&mut self, values: [Float1;3]) { 
		for i in 0..3 {
			self.set_a_element(i, values[i]);
		}
	}
	
	pub fn set_a_element(&mut self, index: usize, value: Float1) { 
		assert!(index<3);
		self.set::<Float1>(0 + index * 16 , value);
	}
	
	pub fn a(&self) ->  [Float1;3] { 
		self.get(0)
	}
	
	pub fn a_element(&self, index: usize) -> Float1 { 
		assert!(index<3);
		self.get::<Float1>(0 + index * 16)
	}
	
	pub fn set_b(&mut self, value: Float2) { 
		self.set(48, value);
	}
	
	pub fn b(&self) -> Float2 { 
		self.get(48)
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

impl Default for LayoutCB {
	fn default() -> Self {
		let mut ret = Self {
		data: [0;64]
		};
		ret.set_a([Float1::default();3]);
		ret.set_b(Float2::default());
		ret
	}
}

