use lgn_data_runtime::ResourceTypeAndId;
use lgn_ecs::prelude::Component;
use lgn_graphics_data::TextureFormat;

#[derive(Component)]
pub struct TextureComponent {
    pub(crate) texture_id: ResourceTypeAndId,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) format: TextureFormat,
    pub(crate) texture_data: Vec<Vec<u8>>,
}

impl TextureComponent {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        texture_id: ResourceTypeAndId,
        width: u32,
        height: u32,
        format: TextureFormat,
        texture_data: Vec<Vec<u8>>,
    ) -> Self {
        Self {
            texture_id,
            width,
            height,
            format,
            texture_data,
        }
    }
}
