// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use lgn_graphics_cgen_runtime::prelude::*;

/*
StructLayout {
    size: 44,
    padded_size: 44,
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
        StructMemberLayout {
            offset: 28,
            absolute_offset: 0,
            size: 4,
            padded_size: 4,
            array_stride: 0,
        },
        StructMemberLayout {
            offset: 32,
            absolute_offset: 0,
            size: 12,
            padded_size: 12,
            array_stride: 0,
        },
    ],
}
*/
static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "Spotlight",
    id: 17,
    size: 44,
};

static_assertions::const_assert_eq!(mem::size_of::<Spotlight>(), 44);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Spotlight {
    data: [u8; 44],
}

impl Spotlight {
    pub const fn id() -> u32 {
        17
    }

    pub fn def() -> &'static CGenTypeDef {
        &TYPE_DEF
    }

    pub fn set_pos(&mut self, value: Float3) {
        self.set(0, value);
    }

    pub fn pos(&self) -> Float3 {
        self.get(0)
    }

    pub fn set_radiance(&mut self, value: Float1) {
        self.set(12, value);
    }

    pub fn radiance(&self) -> Float1 {
        self.get(12)
    }

    pub fn set_dir(&mut self, value: Float3) {
        self.set(16, value);
    }

    pub fn dir(&self) -> Float3 {
        self.get(16)
    }

    pub fn set_cone_angle(&mut self, value: Float1) {
        self.set(28, value);
    }

    pub fn cone_angle(&self) -> Float1 {
        self.get(28)
    }

    pub fn set_color(&mut self, value: Float3) {
        self.set(32, value);
    }

    pub fn color(&self) -> Float3 {
        self.get(32)
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

impl Default for Spotlight {
    fn default() -> Self {
        let mut ret = Self { data: [0; 44] };
        ret.set_pos(Float3::default());
        ret.set_radiance(Float1::default());
        ret.set_dir(Float3::default());
        ret.set_cone_angle(Float1::default());
        ret.set_color(Float3::default());
        ret
    }
}
