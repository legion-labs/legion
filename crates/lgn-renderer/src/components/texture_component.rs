use std::sync::Arc;

use lgn_data_runtime::ResourceTypeAndId;
use lgn_ecs::prelude::Component;
use lgn_graphics_data::TextureFormat;

#[derive(Clone)]
pub struct TextureData {
    data: Arc<Vec<Vec<u8>>>,
}

impl TextureData {
    pub fn from_slice(mip0_data: &[u8]) -> Self {
        Self {
            data: Arc::new(vec![mip0_data.to_owned()]),
        }
    }

    pub fn from_slices(mips_data: &[&[u8]]) -> Self {
        let mut data = Vec::with_capacity(mips_data.len());
        for mip_data in mips_data.iter() {
            data.push((*mip_data).to_owned());
        }

        Self {
            data: Arc::new(data),
        }
    }

    pub fn data(&self) -> &Vec<Vec<u8>> {
        &self.data
    }

    pub fn mip_count(&self) -> usize {
        self.data.len()
    }
}

#[derive(Component)]
pub struct TextureComponent {
    pub(crate) texture_id: ResourceTypeAndId,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) format: TextureFormat,
    pub(crate) texture_data: TextureData,
}

impl TextureComponent {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        texture_id: ResourceTypeAndId,
        width: u32,
        height: u32,
        format: TextureFormat,
        texture_data: TextureData,
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
