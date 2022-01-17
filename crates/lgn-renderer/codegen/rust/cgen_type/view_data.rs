// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "ViewData",
    id: 16,
    size: 416,
};

static_assertions::const_assert_eq!(mem::size_of::<ViewData>(), 416);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ViewData {
    data: [u8; 416],
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl ViewData {
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
    // member : inv_view
    // offset : 64
    // size : 64
    //
    pub fn set_inv_view(&mut self, value: Float4x4) {
        self.set(64, value);
    }

    pub fn inv_view(&self) -> Float4x4 {
        self.get(64)
    }

    //
    // member : projection
    // offset : 128
    // size : 64
    //
    pub fn set_projection(&mut self, value: Float4x4) {
        self.set(128, value);
    }

    pub fn projection(&self) -> Float4x4 {
        self.get(128)
    }

    //
    // member : inv_projection
    // offset : 192
    // size : 64
    //
    pub fn set_inv_projection(&mut self, value: Float4x4) {
        self.set(192, value);
    }

    pub fn inv_projection(&self) -> Float4x4 {
        self.get(192)
    }

    //
    // member : projection_view
    // offset : 256
    // size : 64
    //
    pub fn set_projection_view(&mut self, value: Float4x4) {
        self.set(256, value);
    }

    pub fn projection_view(&self) -> Float4x4 {
        self.get(256)
    }

    //
    // member : inv_projection_view
    // offset : 320
    // size : 64
    //
    pub fn set_inv_projection_view(&mut self, value: Float4x4) {
        self.set(320, value);
    }

    pub fn inv_projection_view(&self) -> Float4x4 {
        self.get(320)
    }

    //
    // member : screen_size
    // offset : 384
    // size : 16
    //
    pub fn set_screen_size(&mut self, value: Float4) {
        self.set(384, value);
    }

    pub fn screen_size(&self) -> Float4 {
        self.get(384)
    }

    //
    // member : cursor_pos
    // offset : 400
    // size : 16
    //
    pub fn set_cursor_pos(&mut self, value: Float2) {
        self.set(400, value);
    }

    pub fn cursor_pos(&self) -> Float2 {
        self.get(400)
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

impl Default for ViewData {
    fn default() -> Self {
        let mut ret = Self { data: [0; 416] };
        ret.set_view(Float4x4::default());
        ret.set_inv_view(Float4x4::default());
        ret.set_projection(Float4x4::default());
        ret.set_inv_projection(Float4x4::default());
        ret.set_projection_view(Float4x4::default());
        ret.set_inv_projection_view(Float4x4::default());
        ret.set_screen_size(Float4::default());
        ret.set_cursor_pos(Float2::default());
        ret
    }
}
