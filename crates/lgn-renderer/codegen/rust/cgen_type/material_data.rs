// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "MaterialData",
    id: 24,
    size: 56,
};

static_assertions::const_assert_eq!(mem::size_of::<MaterialData>(), 56);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MaterialData {
    data: [u8; 56],
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl MaterialData {
    pub const fn id() -> u32 {
        24
    }

    pub fn def() -> &'static CGenTypeDef {
        &TYPE_DEF
    }

    //
    // member : base_color
    // offset : 0
    // size : 16
    //
    pub fn set_base_color(&mut self, value: Float4) {
        self.set(0, value);
    }

    pub fn base_color(&self) -> Float4 {
        self.get(0)
    }

    //
    // member : subsurface
    // offset : 16
    // size : 4
    //
    pub fn set_subsurface(&mut self, value: Float1) {
        self.set(16, value);
    }

    pub fn subsurface(&self) -> Float1 {
        self.get(16)
    }

    //
    // member : metallic
    // offset : 20
    // size : 4
    //
    pub fn set_metallic(&mut self, value: Float1) {
        self.set(20, value);
    }

    pub fn metallic(&self) -> Float1 {
        self.get(20)
    }

    //
    // member : specular
    // offset : 24
    // size : 4
    //
    pub fn set_specular(&mut self, value: Float1) {
        self.set(24, value);
    }

    pub fn specular(&self) -> Float1 {
        self.get(24)
    }

    //
    // member : specular_tint
    // offset : 28
    // size : 4
    //
    pub fn set_specular_tint(&mut self, value: Float1) {
        self.set(28, value);
    }

    pub fn specular_tint(&self) -> Float1 {
        self.get(28)
    }

    //
    // member : roughness
    // offset : 32
    // size : 4
    //
    pub fn set_roughness(&mut self, value: Float1) {
        self.set(32, value);
    }

    pub fn roughness(&self) -> Float1 {
        self.get(32)
    }

    //
    // member : anisotropic
    // offset : 36
    // size : 4
    //
    pub fn set_anisotropic(&mut self, value: Float1) {
        self.set(36, value);
    }

    pub fn anisotropic(&self) -> Float1 {
        self.get(36)
    }

    //
    // member : sheen
    // offset : 40
    // size : 4
    //
    pub fn set_sheen(&mut self, value: Float1) {
        self.set(40, value);
    }

    pub fn sheen(&self) -> Float1 {
        self.get(40)
    }

    //
    // member : sheen_tint
    // offset : 44
    // size : 4
    //
    pub fn set_sheen_tint(&mut self, value: Float1) {
        self.set(44, value);
    }

    pub fn sheen_tint(&self) -> Float1 {
        self.get(44)
    }

    //
    // member : clearcoat
    // offset : 48
    // size : 4
    //
    pub fn set_clearcoat(&mut self, value: Float1) {
        self.set(48, value);
    }

    pub fn clearcoat(&self) -> Float1 {
        self.get(48)
    }

    //
    // member : clearcoat_gloss
    // offset : 52
    // size : 4
    //
    pub fn set_clearcoat_gloss(&mut self, value: Float1) {
        self.set(52, value);
    }

    pub fn clearcoat_gloss(&self) -> Float1 {
        self.get(52)
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

impl Default for MaterialData {
    fn default() -> Self {
        let mut ret = Self { data: [0; 56] };
        ret.set_base_color(Float4::default());
        ret.set_subsurface(Float1::default());
        ret.set_metallic(Float1::default());
        ret.set_specular(Float1::default());
        ret.set_specular_tint(Float1::default());
        ret.set_roughness(Float1::default());
        ret.set_anisotropic(Float1::default());
        ret.set_sheen(Float1::default());
        ret.set_sheen_tint(Float1::default());
        ret.set_clearcoat(Float1::default());
        ret.set_clearcoat_gloss(Float1::default());
        ret
    }
}
