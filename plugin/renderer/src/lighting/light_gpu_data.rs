use crate::cgen::cgen_type::{DirectionalLight, OmnidirectionalLight, Spotlight};

impl OmnidirectionalLight {
    pub const SIZE: usize = 32;
}

impl DirectionalLight {
    pub const SIZE: usize = 32;
}

impl Spotlight {
    pub const SIZE: usize = 64;
}
