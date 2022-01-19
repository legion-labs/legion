// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "PickingPushConstantData",
    id: 22,
    size: 96,
};

static_assertions::const_assert_eq!(mem::size_of::<PickingPushConstantData>(), 96);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct PickingPushConstantData {
    data: [u8; 96],
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
    // member : picking_distance
    // offset : 80
    // size : 4
    //
    pub fn set_picking_distance(&mut self, value: Float1) {
        self.set(80, value);
    }

    pub fn picking_distance(&self) -> Float1 {
        self.get(80)
    }

    //
    // member : vertex_offset
    // offset : 84
    // size : 4
    //
    pub fn set_vertex_offset(&mut self, value: Uint1) {
        self.set(84, value);
    }

    pub fn vertex_offset(&self) -> Uint1 {
        self.get(84)
    }

    //
    // member : world_offset
    // offset : 88
    // size : 4
    //
    pub fn set_world_offset(&mut self, value: Uint1) {
        self.set(88, value);
    }

    pub fn world_offset(&self) -> Uint1 {
        self.get(88)
    }

    //
    // member : picking_id
    // offset : 92
    // size : 4
    //
    pub fn set_picking_id(&mut self, value: Uint1) {
        self.set(92, value);
    }

    pub fn picking_id(&self) -> Uint1 {
        self.get(92)
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
        let mut ret = Self { data: [0; 96] };
        ret.set_world(Float4x4::default());
        ret.set_color(Float4::default());
        ret.set_picking_distance(Float1::default());
        ret.set_vertex_offset(Uint1::default());
        ret.set_world_offset(Uint1::default());
        ret.set_picking_id(Uint1::default());
        ret.set_picking_distance(Float1::default());
        ret
    }
}
