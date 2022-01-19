// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "SpotLight",
    id: 15,
    size: 64,
};

static_assertions::const_assert_eq!(mem::size_of::<SpotLight>(), 64);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct SpotLight {
    data: [u8; 64],
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl SpotLight {
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
    // member : dir
    // offset : 16
    // size : 12
    //
    pub fn set_dir(&mut self, value: Float3) {
        self.set(16, value);
    }

    pub fn dir(&self) -> Float3 {
        self.get(16)
    }

    //
    // member : cone_angle
    // offset : 28
    // size : 4
    //
    pub fn set_cone_angle(&mut self, value: Float1) {
        self.set(28, value);
    }

    pub fn cone_angle(&self) -> Float1 {
        self.get(28)
    }

    //
    // member : color
    // offset : 32
    // size : 12
    //
    pub fn set_color(&mut self, value: Float3) {
        self.set(32, value);
    }

    pub fn color(&self) -> Float3 {
        self.get(32)
    }

    //
    // member : pad
    // offset : 44
    // size : 20
    //
    pub fn set_pad(&mut self, values: [Uint1; 5]) {
        for i in 0..5 {
            self.set_pad_element(i, values[i]);
        }
    }

    pub fn set_pad_element(&mut self, index: usize, value: Uint1) {
        assert!(index < 5);
        self.set::<Uint1>(44 + index * 4, value);
    }

    pub fn pad(&self) -> [Uint1; 5] {
        self.get(44)
    }

    pub fn pad_element(&self, index: usize) -> Uint1 {
        assert!(index < 5);
        self.get::<Uint1>(44 + index * 4)
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

impl Default for SpotLight {
    fn default() -> Self {
        let mut ret = Self { data: [0; 64] };
        ret.set_pos(Float3::default());
        ret.set_radiance(Float1::default());
        ret.set_dir(Float3::default());
        ret.set_cone_angle(Float1::default());
        ret.set_color(Float3::default());
        ret.set_pad([Uint1::default(); 5]);
        ret
    }
}
