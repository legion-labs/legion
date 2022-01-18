// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "PickingPushConstantData",
    id: 22,
    size: 80,
};

static_assertions::const_assert_eq!(mem::size_of::<PickingPushConstantData>(), 80);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct PickingPushConstantData {
    data: [u8; 80],
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
    // member : custom_world
    // offset : 0
    // size : 64
    //
    pub fn set_custom_world(&mut self, value: Float4x4) {
        self.set(0, value);
    }

    pub fn custom_world(&self) -> Float4x4 {
        self.get(0)
    }

    //
    // member : vertex_offset
    // offset : 64
    // size : 4
    //
    pub fn set_vertex_offset(&mut self, value: Uint1) {
        self.set(64, value);
    }

    pub fn vertex_offset(&self) -> Uint1 {
        self.get(64)
    }

    //
    // member : world_offset
    // offset : 68
    // size : 4
    //
    pub fn set_world_offset(&mut self, value: Uint1) {
        self.set(68, value);
    }

    pub fn world_offset(&self) -> Uint1 {
        self.get(68)
    }

    //
    // member : picking_id
    // offset : 72
    // size : 4
    //
    pub fn set_picking_id(&mut self, value: Uint1) {
        self.set(72, value);
    }

    pub fn picking_id(&self) -> Uint1 {
        self.get(72)
    }

    //
    // member : picking_distance
    // offset : 76
    // size : 4
    //
    pub fn set_picking_distance(&mut self, value: Float1) {
        self.set(76, value);
    }

    pub fn picking_distance(&self) -> Float1 {
        self.get(76)
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
        let mut ret = Self { data: [0; 80] };
        ret.set_custom_world(Float4x4::default());
        ret.set_vertex_offset(Uint1::default());
        ret.set_world_offset(Uint1::default());
        ret.set_picking_id(Uint1::default());
        ret.set_picking_distance(Float1::default());
        ret
    }
}
