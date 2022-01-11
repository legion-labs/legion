// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use super::layout_sb::LayoutSB;

/*
StructLayout {
    size: 58,
    padded_size: 58,
    members: [
        StructMemberLayout {
            offset: 0,
            absolute_offset: 0,
            size: 58,
            padded_size: 58,
            array_stride: 0,
        },
    ],
}
*/
static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "LayoutSB2",
    id: 17,
    size: 58,
};

static_assertions::const_assert_eq!(mem::size_of::<LayoutSB2>(), 58);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct LayoutSB2 {
    data: [u8; 58],
}

impl LayoutSB2 {
    pub const fn id() -> u32 {
        17
    }

    pub fn def() -> &'static CGenTypeDef {
        &TYPE_DEF
    }

    pub fn set_a(&mut self, value: LayoutSB) {
        self.set(0, value);
    }

    pub fn a(&self) -> LayoutSB {
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

impl Default for LayoutSB2 {
    fn default() -> Self {
        let mut ret = Self { data: [0; 58] };
        ret.set_a(LayoutSB::default());
        ret
    }
}
