// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "LightingData",
    id: 24,
    size: 36,
};

static_assertions::const_assert_eq!(mem::size_of::<LightingData>(), 36);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct LightingData {
    data: [u8; 36],
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl LightingData {
    pub const fn id() -> u32 {
        24
    }

    pub fn def() -> &'static CGenTypeDef {
        &TYPE_DEF
    }

    //
    // member : num_directional_lights
    // offset : 0
    // size : 4
    //
    pub fn set_num_directional_lights(&mut self, value: Uint1) {
        self.set(0, value);
    }

    pub fn num_directional_lights(&self) -> Uint1 {
        self.get(0)
    }

    //
    // member : num_omnidirectional_lights
    // offset : 4
    // size : 4
    //
    pub fn set_num_omnidirectional_lights(&mut self, value: Uint1) {
        self.set(4, value);
    }

    pub fn num_omnidirectional_lights(&self) -> Uint1 {
        self.get(4)
    }

    //
    // member : num_spotlights
    // offset : 8
    // size : 4
    //
    pub fn set_num_spotlights(&mut self, value: Uint1) {
        self.set(8, value);
    }

    pub fn num_spotlights(&self) -> Uint1 {
        self.get(8)
    }

    //
    // member : diffuse
    // offset : 12
    // size : 4
    //
    pub fn set_diffuse(&mut self, value: Uint1) {
        self.set(12, value);
    }

    pub fn diffuse(&self) -> Uint1 {
        self.get(12)
    }

    //
    // member : specular
    // offset : 16
    // size : 4
    //
    pub fn set_specular(&mut self, value: Uint1) {
        self.set(16, value);
    }

    pub fn specular(&self) -> Uint1 {
        self.get(16)
    }

    //
    // member : specular_reflection
    // offset : 20
    // size : 4
    //
    pub fn set_specular_reflection(&mut self, value: Float1) {
        self.set(20, value);
    }

    pub fn specular_reflection(&self) -> Float1 {
        self.get(20)
    }

    //
    // member : diffuse_reflection
    // offset : 24
    // size : 4
    //
    pub fn set_diffuse_reflection(&mut self, value: Float1) {
        self.set(24, value);
    }

    pub fn diffuse_reflection(&self) -> Float1 {
        self.get(24)
    }

    //
    // member : ambient_reflection
    // offset : 28
    // size : 4
    //
    pub fn set_ambient_reflection(&mut self, value: Float1) {
        self.set(28, value);
    }

    pub fn ambient_reflection(&self) -> Float1 {
        self.get(28)
    }

    //
    // member : shininess
    // offset : 32
    // size : 4
    //
    pub fn set_shininess(&mut self, value: Float1) {
        self.set(32, value);
    }

    pub fn shininess(&self) -> Float1 {
        self.get(32)
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

impl Default for LightingData {
    fn default() -> Self {
        let mut ret = Self { data: [0; 36] };
        ret.set_num_directional_lights(Uint1::default());
        ret.set_num_omnidirectional_lights(Uint1::default());
        ret.set_num_spotlights(Uint1::default());
        ret.set_diffuse(Uint1::default());
        ret.set_specular(Uint1::default());
        ret.set_specular_reflection(Float1::default());
        ret.set_diffuse_reflection(Float1::default());
        ret.set_ambient_reflection(Float1::default());
        ret.set_shininess(Float1::default());
        ret
    }
}
