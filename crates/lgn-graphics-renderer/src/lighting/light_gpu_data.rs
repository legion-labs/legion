use crate::cgen::cgen_type::{DirectionalLight, OmniDirectionalLight, SpotLight};

impl OmniDirectionalLight {
    pub const NUM: u64 = 4096;
}

impl DirectionalLight {
    pub const NUM: u64 = 4096;
}

impl SpotLight {
    pub const NUM: u64 = 4096;
}
