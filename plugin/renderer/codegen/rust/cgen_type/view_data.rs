// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "ViewData",
    id: 21,
    size: 128,
};

static_assertions::const_assert_eq!(mem::size_of::<ViewData>(), 128);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ViewData {
    data: [u8; 128],
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl ViewData {
    pub const fn id() -> u32 {
        21
    }

    pub fn def() -> &'static CGenTypeDef {
        &TYPE_DEF
    }

    //
    // member : view
    // offset : 0
    // size : 64
    //
    pub fn set_view(&mut self, value: Float4x4) {
        self.set(0, value);
    }

    pub fn view(&self) -> Float4x4 {
        self.get(0)
    }

    //
    // member : projection
    // offset : 64
    // size : 64
    //
    pub fn set_projection(&mut self, value: Float4x4) {
        self.set(64, value);
    }

    pub fn projection(&self) -> Float4x4 {
        self.get(64)
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

impl Default for ViewData {
    fn default() -> Self {
        let mut ret = Self { data: [0; 128] };
        ret.set_view(Float4x4::default());
        ret.set_projection(Float4x4::default());
        ret
    }
}
