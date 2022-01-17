// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "PickingPushConstantData",
    id: 22,
    size: 12,
};

static_assertions::const_assert_eq!(mem::size_of::<PickingPushConstantData>(), 12);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct PickingPushConstantData {
    data: [u8; 12],
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl PickingPushConstantData {
    pub const fn id() -> u32 {
        22
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
    // member : picking_id
    // offset : 8
    // size : 4
    //
    pub fn set_picking_id(&mut self, value: Uint1) {
        self.set(8, value);
    }

    pub fn picking_id(&self) -> Uint1 {
        self.get(8)
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

impl Default for PickingPushConstantData {
    fn default() -> Self {
        let mut ret = Self { data: [0; 12] };
        ret.set_vertex_offset(Uint1::default());
        ret.set_world_offset(Uint1::default());
        ret.set_picking_id(Uint1::default());
        ret
    }
}
