// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "LayoutD",
    id: 13,
    size: 16,
};

static_assertions::const_assert_eq!(mem::size_of::<LayoutD>(), 16);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct LayoutD {
    data: [u8; 16],
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl LayoutD {
    pub const fn id() -> u32 {
        13
    }

    pub fn def() -> &'static CGenTypeDef {
        &TYPE_DEF
    }

    //
    // member : a
    // offset : 0
    // size : 16
    //
    pub fn set_a(&mut self, value: Float4) {
        self.set(0, value);
    }

    pub fn a(&self) -> Float4 {
        self.get(0)
    }

    #[allow(unsafe_code)]
    fn set<T: Copy>(&mut self, offset: usize, value: T) {
        unsafe {
            let p = self.data.as_mut_ptr();
            let p = p.add(offset as usize);
            let p = p.cast::<T>();
            p.write(value);
        }
    }

    #[allow(unsafe_code)]
    fn get<T: Copy>(&self, offset: usize) -> T {
        unsafe {
            let p = self.data.as_ptr();
            let p = p.add(offset as usize);
            let p = p.cast::<T>();
            *p
        }
    }
}

impl Default for LayoutD {
    fn default() -> Self {
        let mut ret = Self { data: [0; 16] };
        ret.set_a(Float4::default());
        ret
    }
}
