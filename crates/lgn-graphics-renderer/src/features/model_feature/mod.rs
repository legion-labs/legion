use super::RenderFeature;

pub struct ModelFeature {}

impl ModelFeature {
    pub fn new() -> Self {
        Self {}
    }
}

impl RenderFeature for ModelFeature {}
