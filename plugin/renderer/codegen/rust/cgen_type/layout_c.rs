// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "LayoutC",
    id: 14,
    size: 4,
};

static_assertions::const_assert_eq!(mem::size_of::<LayoutC>(), 4);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct LayoutC {
    data: [u8; 4],
}

impl LayoutC {
    pub const fn id() -> u32 {
        14
    }

    pub fn def() -> &'static CGenTypeDef {
        &TYPE_DEF
    }

    //
    // member : a
    // offset : 0
    // size : 4
    //
    pub fn set_a(&mut self, value: Float1) {
        self.set(0, value);
    }

    pub fn a(&self) -> Float1 {
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

impl Default for LayoutC {
    fn default() -> Self {
        let mut ret = Self { data: [0; 4] };
        ret.set_a(Float1::default());
        ret
    }
}
