// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "InstancePushConstantData",
    id: 23,
    size: 32,
};

static_assertions::const_assert_eq!(mem::size_of::<InstancePushConstantData>(), 32);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct InstancePushConstantData {
    data: [u8; 32],
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl InstancePushConstantData {
    pub const fn id() -> u32 {
        23
    }

    pub fn def() -> &'static CGenTypeDef {
        &TYPE_DEF
    }

    //
    // member : vertex_offset
    // offset : 0
    // size : 4
    //
    pub fn set_vertex_offset(&mut self, value: Uint1) {
        self.set(0, value);
    }

    pub fn vertex_offset(&self) -> Uint1 {
        self.get(0)
    }

    //
    // member : world_offset
    // offset : 4
    // size : 4
    //
    pub fn set_world_offset(&mut self, value: Uint1) {
        self.set(4, value);
    }

    pub fn world_offset(&self) -> Uint1 {
        self.get(4)
    }

    //
    // member : is_picked
    // offset : 8
    // size : 4
    //
    pub fn set_is_picked(&mut self, value: Uint1) {
        self.set(8, value);
    }

    pub fn is_picked(&self) -> Uint1 {
        self.get(8)
    }

    //
    // member : color
    // offset : 16
    // size : 16
    //
    pub fn set_color(&mut self, value: Float4) {
        self.set(16, value);
    }

    pub fn color(&self) -> Float4 {
        self.get(16)
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

impl Default for InstancePushConstantData {
    fn default() -> Self {
        let mut ret = Self { data: [0; 32] };
        ret.set_vertex_offset(Uint1::default());
        ret.set_world_offset(Uint1::default());
        ret.set_is_picked(Uint1::default());
        ret.set_color(Float4::default());
        ret
    }
}
