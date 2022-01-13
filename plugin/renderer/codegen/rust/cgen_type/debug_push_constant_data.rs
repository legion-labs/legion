// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "DebugPushConstantData",
    id: 21,
    size: 4,
};

static_assertions::const_assert_eq!(mem::size_of::<DebugPushConstantData>(), 4);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct DebugPushConstantData {
    data: [u8; 4],
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl DebugPushConstantData {
    pub const fn id() -> u32 {
        21
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

impl Default for DebugPushConstantData {
    fn default() -> Self {
        let mut ret = Self { data: [0; 4] };
        ret.set_vertex_offset(Uint1::default());
        ret
    }
}
