// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "EntityTransforms",
    id: 17,
    size: 64,
};

static_assertions::const_assert_eq!(mem::size_of::<EntityTransforms>(), 64);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct EntityTransforms {
    data: [u8; 64],
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl EntityTransforms {
    pub const fn id() -> u32 {
        17
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

impl Default for EntityTransforms {
    fn default() -> Self {
        let mut ret = Self { data: [0; 64] };
        ret.set_world(Float4x4::default());
        ret
    }
}
