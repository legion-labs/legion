// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "CameraProps",
    id: 16,
    size: 192,
};

static_assertions::const_assert_eq!(mem::size_of::<CameraProps>(), 192);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CameraProps {
    data: [u8; 192],
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl CameraProps {
    pub const fn id() -> u32 {
        16
    }

    pub fn def() -> &'static CGenTypeDef {
        &TYPE_DEF
    }

    //
    // member : view
    // offset : 0
    // size : 64
    //
    pub fn set_view(&mut self, value: Float4x4) {
        self.set(0, value);
    }

    pub fn view(&self) -> Float4x4 {
        self.get(0)
    }

    //
    // member : projection
    // offset : 64
    // size : 64
    //
    pub fn set_projection(&mut self, value: Float4x4) {
        self.set(64, value);
    }

    pub fn projection(&self) -> Float4x4 {
        self.get(64)
    }

    //
    // member : projection_view
    // offset : 128
    // size : 64
    //
    pub fn set_projection_view(&mut self, value: Float4x4) {
        self.set(128, value);
    }

    pub fn projection_view(&self) -> Float4x4 {
        self.get(128)
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

impl Default for CameraProps {
    fn default() -> Self {
        let mut ret = Self { data: [0; 192] };
        ret.set_view(Float4x4::default());
        ret.set_projection(Float4x4::default());
        ret.set_projection_view(Float4x4::default());
        ret
    }
}
