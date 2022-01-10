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
            array_stride: 0,
        },
        StructMemberLayout {
            offset: 64,
            absolute_offset: 64,
            size: 64,
            padded_size: 64,
            array_stride: 0,
        },
    ],
}
*/
static TYPE_DEF: CGenTypeDef = CGenTypeDef{ 
	name: "ViewData",
	id: 18,
	size: 128,
}; 

static_assertions::const_assert_eq!(mem::size_of::<ViewData>(), 128);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ViewData {
	data: [u8;128]
}

impl ViewData {
	pub const fn id() -> u32 { 18  }
	
	pub fn def() -> &'static CGenTypeDef { &TYPE_DEF }
	
	pub fn set_view(&mut self, value: Float4x4) { 
		self.set(0, value);
	}
	
	pub fn view(&self) -> Float4x4 { 
		self.get(0)
	}
	
	pub fn set_projection(&mut self, value: Float4x4) { 
		self.set(64, value);
	}
	
	pub fn projection(&self) -> Float4x4 { 
		self.get(64)
	}
	
	#[allow(unsafe_code)]
	fn set<T: Copy>(&mut self, offset: usize, value: T) {
		unsafe{
			let p = self.data.as_mut_ptr();
			let p = p.add(offset as usize);
			let p = p.cast::<T>();
			p.write(value);
		}
	}
	
	#[allow(unsafe_code)]
	fn get<T: Copy>(&self, offset: usize) -> T {
		unsafe{
			let p = self.data.as_ptr();
			let p = p.add(offset as usize);
			let p = p.cast::<T>();
			*p
		}
	}
}

impl Default for ViewData {
	fn default() -> Self {
		let mut ret = Self {
		data: [0;128]
		};
		ret.set_view(Float4x4::default());
		ret.set_projection(Float4x4::default());
		ret
	}
}

