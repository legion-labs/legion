// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::{
	CGenTypeDef,
};

use lgn_graphics_cgen_runtime::prelude::*;

/*
StructLayout {
    size: 54,
    padded_size: 54,
    members: [
        StructMemberLayout {
            offset: 0,
            absolute_offset: 0,
            size: 12,
            padded_size: 12,
            array_stride: 4,
        },
        StructMemberLayout {
            offset: 16,
            absolute_offset: 0,
            size: 8,
            padded_size: 8,
            array_stride: 0,
        },
        StructMemberLayout {
            offset: 24,
            absolute_offset: 0,
            size: 16,
            padded_size: 16,
            array_stride: 8,
        },
        StructMemberLayout {
            offset: 40,
            absolute_offset: 0,
            size: 8,
            padded_size: 8,
            array_stride: 8,
        },
        StructMemberLayout {
            offset: 48,
            absolute_offset: 0,
            size: 2,
            padded_size: 2,
            array_stride: 2,
        },
        StructMemberLayout {
            offset: 50,
            absolute_offset: 0,
            size: 4,
            padded_size: 4,
            array_stride: 2,
        },
    ],
}
*/
static TYPE_DEF: CGenTypeDef = CGenTypeDef{ 
	name: "LayoutSB",
	id: 16,
	size: 54,
}; 

static_assertions::const_assert_eq!(mem::size_of::<LayoutSB>(), 54);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct LayoutSB {
	data: [u8;54]
}

impl LayoutSB {
	pub const fn id() -> u32 { 16  }
	
	pub fn def() -> &'static CGenTypeDef { &TYPE_DEF }
	
	pub fn set_a(&mut self, values: [Float1;3]) { 
		for i in 0..3 {
			self.set_a_element(i, values[i]);
		}
	}
	
	pub fn set_a_element(&mut self, index: usize, value: Float1) { 
		assert!(index<3);
		self.set::<Float1>(0 + index * 4 , value);
	}
	
	pub fn a(&self) ->  [Float1;3] { 
		self.get(0)
	}
	
	pub fn a_element(&self, index: usize) -> Float1 { 
		assert!(index<3);
		self.get::<Float1>(0 + index * 4)
	}
	
	pub fn set_b(&mut self, value: Float2) { 
		self.set(16, value);
	}
	
	pub fn b(&self) -> Float2 { 
		self.get(16)
	}
	
	pub fn set_c(&mut self, values: [Uint2;2]) { 
		for i in 0..2 {
			self.set_c_element(i, values[i]);
		}
	}
	
	pub fn set_c_element(&mut self, index: usize, value: Uint2) { 
		assert!(index<2);
		self.set::<Uint2>(24 + index * 8 , value);
	}
	
	pub fn c(&self) ->  [Uint2;2] { 
		self.get(24)
	}
	
	pub fn c_element(&self, index: usize) -> Uint2 { 
		assert!(index<2);
		self.get::<Uint2>(24 + index * 8)
	}
	
	pub fn set_d(&mut self, values: [Half3;1]) { 
		for i in 0..1 {
			self.set_d_element(i, values[i]);
		}
	}
	
	pub fn set_d_element(&mut self, index: usize, value: Half3) { 
		assert!(index<1);
		self.set::<Half3>(40 + index * 8 , value);
	}
	
	pub fn d(&self) ->  [Half3;1] { 
		self.get(40)
	}
	
	pub fn d_element(&self, index: usize) -> Half3 { 
		assert!(index<1);
		self.get::<Half3>(40 + index * 8)
	}
	
	pub fn set_e(&mut self, values: [Half1;1]) { 
		for i in 0..1 {
			self.set_e_element(i, values[i]);
		}
	}
	
	pub fn set_e_element(&mut self, index: usize, value: Half1) { 
		assert!(index<1);
		self.set::<Half1>(48 + index * 2 , value);
	}
	
	pub fn e(&self) ->  [Half1;1] { 
		self.get(48)
	}
	
	pub fn e_element(&self, index: usize) -> Half1 { 
		assert!(index<1);
		self.get::<Half1>(48 + index * 2)
	}
	
	pub fn set_f(&mut self, values: [Half1;2]) { 
		for i in 0..2 {
			self.set_f_element(i, values[i]);
		}
	}
	
	pub fn set_f_element(&mut self, index: usize, value: Half1) { 
		assert!(index<2);
		self.set::<Half1>(50 + index * 2 , value);
	}
	
	pub fn f(&self) ->  [Half1;2] { 
		self.get(50)
	}
	
	pub fn f_element(&self, index: usize) -> Half1 { 
		assert!(index<2);
		self.get::<Half1>(50 + index * 2)
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

impl Default for LayoutSB {
	fn default() -> Self {
		let mut ret = Self {
		data: [0;54]
		};
		ret.set_a([Float1::default();3]);
		ret.set_b(Float2::default());
		ret.set_c([Uint2::default();2]);
		ret.set_d([Half3::default();1]);
		ret.set_e([Half1::default();1]);
		ret.set_f([Half1::default();2]);
		ret
	}
}

