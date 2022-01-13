// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "OmnidirectionalLight",
    id: 15,
    size: 32,
};

static_assertions::const_assert_eq!(mem::size_of::<OmnidirectionalLight>(), 32);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct OmnidirectionalLight {
    data: [u8; 32],
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl OmnidirectionalLight {
    pub const fn id() -> u32 {
        15
    }

    pub fn def() -> &'static CGenTypeDef {
        &TYPE_DEF
    }

    //
    // member : pos
    // offset : 0
    // size : 12
    //
    pub fn set_pos(&mut self, value: Float3) {
        self.set(0, value);
    }

    pub fn pos(&self) -> Float3 {
        self.get(0)
    }

    //
    // member : radiance
    // offset : 12
    // size : 4
    //
    pub fn set_radiance(&mut self, value: Float1) {
        self.set(12, value);
    }

    pub fn radiance(&self) -> Float1 {
        self.get(12)
    }

    //
    // member : color
    // offset : 16
    // size : 12
    //
    pub fn set_color(&mut self, value: Float3) {
        self.set(16, value);
    }

    pub fn color(&self) -> Float3 {
        self.get(16)
    }

    //
    // member : pad
    // offset : 28
    // size : 4
    //
    pub fn set_pad(&mut self, value: Uint1) {
        self.set(28, value);
    }

    pub fn pad(&self) -> Uint1 {
        self.get(28)
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

impl Default for OmnidirectionalLight {
    fn default() -> Self {
        let mut ret = Self { data: [0; 32] };
        ret.set_pos(Float3::default());
        ret.set_radiance(Float1::default());
        ret.set_color(Float3::default());
        ret.set_pad(Uint1::default());
        ret
    }
}
