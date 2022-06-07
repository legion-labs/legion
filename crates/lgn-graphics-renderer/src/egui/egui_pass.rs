use lgn_graphics_api::prelude::*;

// TODO(jsg): Move this somewhere else to be able to remove this struct entirely.
pub struct EguiPass {
    pub(crate) texture_data: Option<(u64, Texture, TextureView)>,
}

impl EguiPass {
    pub fn new() -> Self {
        Self { texture_data: None }
    }
}
