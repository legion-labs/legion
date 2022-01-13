// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "EguiPushConstantData",
    id: 20,
    size: 24,
};

static_assertions::const_assert_eq!(mem::size_of::<EguiPushConstantData>(), 24);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct EguiPushConstantData {
    data: [u8; 24],
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl EguiPushConstantData {
    pub const fn id() -> u32 {
        20
    }

    pub fn def() -> &'static CGenTypeDef {
        &TYPE_DEF
    }

    //
    // member : scale
    // offset : 0
    // size : 8
    //
    pub fn set_scale(&mut self, value: Float2) {
        self.set(0, value);
    }

    pub fn scale(&self) -> Float2 {
        self.get(0)
    }

    //
    // member : translation
    // offset : 8
    // size : 8
    //
    pub fn set_translation(&mut self, value: Float2) {
        self.set(8, value);
    }

    pub fn translation(&self) -> Float2 {
        self.get(8)
    }

    //
    // member : width
    // offset : 16
    // size : 4
    //
    pub fn set_width(&mut self, value: Float1) {
        self.set(16, value);
    }

    pub fn width(&self) -> Float1 {
        self.get(16)
    }

    //
    // member : height
    // offset : 20
    // size : 4
    //
    pub fn set_height(&mut self, value: Float1) {
        self.set(20, value);
    }

    pub fn height(&self) -> Float1 {
        self.get(20)
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

impl Default for EguiPushConstantData {
    fn default() -> Self {
        let mut ret = Self { data: [0; 24] };
        ret.set_scale(Float2::default());
        ret.set_translation(Float2::default());
        ret.set_width(Float1::default());
        ret.set_height(Float1::default());
        ret
    }
}
