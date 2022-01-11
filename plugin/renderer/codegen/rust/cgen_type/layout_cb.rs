// This is generated file. Do not edit manually

use std::mem;

use lgn_graphics_cgen_runtime::CGenTypeDef;

use lgn_graphics_cgen_runtime::prelude::*;

static TYPE_DEF: CGenTypeDef = CGenTypeDef {
    name: "LayoutCB",
    id: 18,
    size: 176,
};

static_assertions::const_assert_eq!(mem::size_of::<LayoutCB>(), 176);

#[derive(Clone, Copy)]
#[repr(C)]
pub struct LayoutCB {
    data: [u8; 176],
}

impl LayoutCB {
    pub const fn id() -> u32 {
        18
    }

    pub fn def() -> &'static CGenTypeDef {
        &TYPE_DEF
    }

    //
    // member : a
    // offset : 0
    // size : 48
    //
    pub fn set_a(&mut self, values: [Float1; 3]) {
        for i in 0..3 {
            self.set_a_element(i, values[i]);
        }
    }

    pub fn set_a_element(&mut self, index: usize, value: Float1) {
        assert!(index < 3);
        self.set::<Float1>(0 + index * 16, value);
    }

    pub fn a(&self) -> [Float1; 3] {
        self.get(0)
    }

    pub fn a_element(&self, index: usize) -> Float1 {
        assert!(index < 3);
        self.get::<Float1>(0 + index * 16)
    }

    //
    // member : b
    // offset : 48
    // size : 16
    //
    pub fn set_b(&mut self, value: Float2) {
        self.set(48, value);
    }

    pub fn b(&self) -> Float2 {
        self.get(48)
    }

    //
    // member : c
    // offset : 64
    // size : 32
    //
    pub fn set_c(&mut self, values: [Uint2; 2]) {
        for i in 0..2 {
            self.set_c_element(i, values[i]);
        }
    }

    pub fn set_c_element(&mut self, index: usize, value: Uint2) {
        assert!(index < 2);
        self.set::<Uint2>(64 + index * 16, value);
    }

    pub fn c(&self) -> [Uint2; 2] {
        self.get(64)
    }

    pub fn c_element(&self, index: usize) -> Uint2 {
        assert!(index < 2);
        self.get::<Uint2>(64 + index * 16)
    }

    //
    // member : d
    // offset : 96
    // size : 16
    //
    pub fn set_d(&mut self, values: [Half3; 1]) {
        for i in 0..1 {
            self.set_d_element(i, values[i]);
        }
    }

    pub fn set_d_element(&mut self, index: usize, value: Half3) {
        assert!(index < 1);
        self.set::<Half3>(96 + index * 16, value);
    }

    pub fn d(&self) -> [Half3; 1] {
        self.get(96)
    }

    pub fn d_element(&self, index: usize) -> Half3 {
        assert!(index < 1);
        self.get::<Half3>(96 + index * 16)
    }

    //
    // member : e
    // offset : 112
    // size : 16
    //
    pub fn set_e(&mut self, values: [Half1; 1]) {
        for i in 0..1 {
            self.set_e_element(i, values[i]);
        }
    }

    pub fn set_e_element(&mut self, index: usize, value: Half1) {
        assert!(index < 1);
        self.set::<Half1>(112 + index * 16, value);
    }

    pub fn e(&self) -> [Half1; 1] {
        self.get(112)
    }

    pub fn e_element(&self, index: usize) -> Half1 {
        assert!(index < 1);
        self.get::<Half1>(112 + index * 16)
    }

    //
    // member : f
    // offset : 128
    // size : 32
    //
    pub fn set_f(&mut self, values: [Half1; 2]) {
        for i in 0..2 {
            self.set_f_element(i, values[i]);
        }
    }

    pub fn set_f_element(&mut self, index: usize, value: Half1) {
        assert!(index < 2);
        self.set::<Half1>(128 + index * 16, value);
    }

    pub fn f(&self) -> [Half1; 2] {
        self.get(128)
    }

    pub fn f_element(&self, index: usize) -> Half1 {
        assert!(index < 2);
        self.get::<Half1>(128 + index * 16)
    }

    //
    // member : g
    // offset : 160
    // size : 16
    //
    pub fn set_g(&mut self, value: Half1) {
        self.set(160, value);
    }

    pub fn g(&self) -> Half1 {
        self.get(160)
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

impl Default for LayoutCB {
    fn default() -> Self {
        let mut ret = Self { data: [0; 176] };
        ret.set_a([Float1::default(); 3]);
        ret.set_b(Float2::default());
        ret.set_c([Uint2::default(); 2]);
        ret.set_d([Half3::default(); 1]);
        ret.set_e([Half1::default(); 1]);
        ret.set_f([Half1::default(); 2]);
        ret.set_g(Half1::default());
        ret
    }
}
