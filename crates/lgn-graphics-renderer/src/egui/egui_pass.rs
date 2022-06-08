use lgn_graphics_api::prelude::*;

// TODO(jsg): Move this somewhere else to be able to remove this struct entirely.
pub struct EguiPass {
    pub(crate) font_texture: Option<(Texture, TextureView)>,
}

impl EguiPass {
    pub fn new() -> Self {
        Self { font_texture: None }
    }
}
