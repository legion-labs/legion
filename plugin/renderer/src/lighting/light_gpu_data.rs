use crate::cgen::cgen_type::{DirectionalLight, OmnidirectionalLight, Spotlight};

impl OmnidirectionalLight {
    pub const SIZE: u32 = std::mem::size_of::<Self>() as u32;
    pub const NUM: u32 = 4096;
    pub const PAGE_SIZE: u64 = (Self::SIZE * Self::NUM) as u64;
}

impl DirectionalLight {
    pub const SIZE: u32 = std::mem::size_of::<Self>() as u32;
    pub const NUM: u32 = 4096;
    pub const PAGE_SIZE: u64 = (Self::SIZE * Self::NUM) as u64;
}

impl Spotlight {
    pub const SIZE: u32 = std::mem::size_of::<Self>() as u32;
    pub const NUM: u32 = 4096;
    pub const PAGE_SIZE: u64 = (Self::SIZE * Self::NUM) as u64;
}
