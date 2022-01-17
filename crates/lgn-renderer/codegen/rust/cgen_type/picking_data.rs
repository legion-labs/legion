// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "PickingData",
    id: 23,
    size: 16,
};

static_assertions::const_assert_eq!(mem::size_of::<PickingData>(), 16);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct PickingData {
    data: [u8; 16],
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl PickingData {
    pub const fn id() -> u32 {
        23
    }

    pub fn def() -> &'static CGenTypeDef {
        &TYPE_DEF
    }

    //
    // member : picking_pos
    // offset : 0
    // size : 12
    //
    pub fn set_picking_pos(&mut self, value: Float3) {
        self.set(0, value);
    }

    pub fn picking_pos(&self) -> Float3 {
        self.get(0)
    }

    //
    // member : picking_id
    // offset : 12
    // size : 4
    //
    pub fn set_picking_id(&mut self, value: Uint1) {
        self.set(12, value);
    }

    pub fn picking_id(&self) -> Uint1 {
        self.get(12)
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

impl Default for PickingData {
    fn default() -> Self {
        let mut ret = Self { data: [0; 16] };
        ret.set_picking_pos(Float3::default());
        ret.set_picking_id(Uint1::default());
        ret
    }
}
