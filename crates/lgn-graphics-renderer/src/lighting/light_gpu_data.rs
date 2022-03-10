use crate::cgen::cgen_type::{DirectionalLight, OmniDirectionalLight, SpotLight};

impl OmniDirectionalLight {
    pub const SIZE: u32 = std::mem::size_of::<Self>() as u32;
    pub const NUM: u32 = 4096;
    pub const PAGE_SIZE: u64 = (Self::SIZE * Self::NUM) as u64;
}

impl DirectionalLight {
    pub const SIZE: u32 = std::mem::size_of::<Self>() as u32;
    pub const NUM: u32 = 4096;
    pub const PAGE_SIZE: u64 = (Self::SIZE * Self::NUM) as u64;
}

impl SpotLight {
    pub const SIZE: u32 = std::mem::size_of::<Self>() as u32;
    pub const NUM: u32 = 4096;
    pub const PAGE_SIZE: u64 = (Self::SIZE * Self::NUM) as u64;
}
