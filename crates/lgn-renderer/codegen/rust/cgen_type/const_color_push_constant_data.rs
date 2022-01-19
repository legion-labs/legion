// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "ConstColorPushConstantData",
    id: 21,
    size: 84,
};

static_assertions::const_assert_eq!(mem::size_of::<ConstColorPushConstantData>(), 84);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ConstColorPushConstantData {
    data: [u8; 84],
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl ConstColorPushConstantData {
    pub const fn id() -> u32 {
        21
    }

    pub fn def() -> &'static CGenTypeDef {
        &TYPE_DEF
    }

    //
    // member : world
    // offset : 0
    // size : 64
    //
    pub fn set_world(&mut self, value: Float4x4) {
        self.set(0, value);
    }

    pub fn world(&self) -> Float4x4 {
        self.get(0)
    }

    //
    // member : color
    // offset : 64
    // size : 16
    //
    pub fn set_color(&mut self, value: Float4) {
        self.set(64, value);
    }

    pub fn color(&self) -> Float4 {
        self.get(64)
    }

    //
    // member : vertex_offset
    // offset : 80
    // size : 4
    //
    pub fn set_vertex_offset(&mut self, value: Uint1) {
        self.set(80, value);
    }

    pub fn vertex_offset(&self) -> Uint1 {
        self.get(80)
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

impl Default for ConstColorPushConstantData {
    fn default() -> Self {
        let mut ret = Self { data: [0; 84] };
        ret.set_world(Float4x4::default());
        ret.set_color(Float4::default());
        ret.set_vertex_offset(Uint1::default());
        ret
    }
}
