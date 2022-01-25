// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "MaterialData",
    id: 24,
    size: 28,
};

static_assertions::const_assert_eq!(mem::size_of::<MaterialData>(), 28);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MaterialData {
    data: [u8; 28],
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
    // member : metallic
    // offset : 16
    // size : 4
    //
    pub fn set_metallic(&mut self, value: Float1) {
        self.set(16, value);
    }

    pub fn metallic(&self) -> Float1 {
        self.get(16)
    }

    //
    // member : reflectance
    // offset : 20
    // size : 4
    //
    pub fn set_reflectance(&mut self, value: Float1) {
        self.set(20, value);
    }

    pub fn reflectance(&self) -> Float1 {
        self.get(20)
    }

    //
    // member : roughness
    // offset : 24
    // size : 4
    //
    pub fn set_roughness(&mut self, value: Float1) {
        self.set(24, value);
    }

    pub fn roughness(&self) -> Float1 {
        self.get(24)
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
        let mut ret = Self { data: [0; 28] };
        ret.set_base_color(Float4::default());
        ret.set_metallic(Float1::default());
        ret.set_reflectance(Float1::default());
        ret.set_roughness(Float1::default());
        ret
    }
}
