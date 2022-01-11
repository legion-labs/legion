// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use lgn_graphics_cgen_runtime::prelude::*;

/*
StructLayout {
    size: 28,
    padded_size: 28,
    members: [
        StructMemberLayout {
            offset: 0,
            absolute_offset: 0,
            size: 12,
            padded_size: 12,
            array_stride: 0,
        },
        StructMemberLayout {
            offset: 12,
            absolute_offset: 0,
            size: 4,
            padded_size: 4,
            array_stride: 0,
        },
        StructMemberLayout {
            offset: 16,
            absolute_offset: 0,
            size: 12,
            padded_size: 12,
            array_stride: 0,
        },
    ],
}
*/
static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "DirectionalLight",
    id: 16,
    size: 28,
};

static_assertions::const_assert_eq!(mem::size_of::<DirectionalLight>(), 28);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct DirectionalLight {
    data: [u8; 28],
}

impl DirectionalLight {
    pub const fn id() -> u32 {
        16
    }

    pub fn def() -> &'static CGenTypeDef {
        &TYPE_DEF
    }

    pub fn set_dir(&mut self, value: Float3) {
        self.set(0, value);
    }

    pub fn dir(&self) -> Float3 {
        self.get(0)
    }

    pub fn set_radiance(&mut self, value: Float1) {
        self.set(12, value);
    }

    pub fn radiance(&self) -> Float1 {
        self.get(12)
    }

    pub fn set_color(&mut self, value: Float3) {
        self.set(16, value);
    }

    pub fn color(&self) -> Float3 {
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

impl Default for DirectionalLight {
    fn default() -> Self {
        let mut ret = Self { data: [0; 28] };
        ret.set_dir(Float3::default());
        ret.set_radiance(Float1::default());
        ret.set_color(Float3::default());
        ret
    }
}
