use strum::{EnumCount, IntoEnumIterator};

use lgn_graphics_api::{
    Extents3D, Format, MemoryUsage, ResourceFlags, ResourceUsage, TextureDef, TextureTiling,
};

use crate::{components::TextureData, Renderer};

use super::{GpuTextureId, PersistentDescriptorSetManager, TextureManager};

#[derive(Clone, Copy, strum::EnumCount, strum::EnumIter)]
pub enum SharedTextureId {
    Albedo,
    Normal,
    Metalness,
    Roughness,
}

#[derive(Clone, Copy)]
struct SharedTexture {
    gpu_texture_id: GpuTextureId,
    bindless_index: u32,
}

impl Default for SharedTexture {
    fn default() -> Self {
        Self {
            gpu_texture_id: GpuTextureId::default(),
            bindless_index: u32::MAX,
        }
    }
}

pub struct SharedResourcesManager {
    textures: [SharedTexture; SharedTextureId::COUNT],
}

impl SharedResourcesManager {
    pub fn new(
        renderer: &Renderer,
        texture_manager: &mut TextureManager,
        persistent_descriptor_set_manager: &mut PersistentDescriptorSetManager,
    ) -> Self {
        let mut shared_textures = [SharedTexture::default(); SharedTextureId::COUNT];

        for (index, shared_texture_id) in SharedTextureId::iter().enumerate() {
            let gpu_texture_id = Self::create_texture(texture_manager, shared_texture_id);
            shared_textures[index].gpu_texture_id = gpu_texture_id;
        }

        texture_manager.update(renderer, persistent_descriptor_set_manager);

        for shared_texture in &mut shared_textures {
            let gpu_texture_id = shared_texture.gpu_texture_id;
            shared_texture.bindless_index =
                texture_manager.get_bindless_index(gpu_texture_id).unwrap();
        }

        Self {
            textures: shared_textures,
        }
    }

    pub fn default_texture_bindless_index(&self, shared_texture_id: SharedTextureId) -> u32 {
        self.textures[shared_texture_id as usize].bindless_index
    }

    fn create_texture(
        texture_manager: &mut TextureManager,
        shared_texture_id: SharedTextureId,
    ) -> GpuTextureId {
        let (texture_def, texture_data) = match shared_texture_id {
            SharedTextureId::Albedo => Self::create_albedo_texture(),
            SharedTextureId::Normal => Self::create_normal_texture(), // Self::create_normal_texture(),
            SharedTextureId::Metalness => Self::create_metalness_texture(), // Self::create_metallness_texture(),
            SharedTextureId::Roughness => Self::create_roughness_texture(), // Self::create_roughness_texture(),
        };
        texture_manager.allocate_texture(&texture_def, &texture_data)
    }

    fn create_albedo_texture() -> (TextureDef, TextureData) {
        let texture_def = TextureDef {
            extents: Extents3D {
                width: 4,
                height: 4,
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::R8G8B8A8_UNORM,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Linear,
        };

        let mut texture_data = Vec::<u8>::with_capacity(64);
        for _index in 0..16 {
            texture_data.push(0);
            texture_data.push(0);
            texture_data.push(0);
            texture_data.push(255);
        }

        (texture_def, TextureData::from_slice(&texture_data))
    }

    fn create_normal_texture() -> (TextureDef, TextureData) {
        let texture_def = TextureDef {
            extents: Extents3D {
                width: 4,
                height: 4,
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::R8G8B8A8_UNORM,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Linear,
        };

        let mut texture_data = Vec::<u8>::with_capacity(64);
        for _index in 0..16 {
            texture_data.push(0);
            texture_data.push(0);
            texture_data.push(0);
            texture_data.push(255);
        }

        (texture_def, TextureData::from_slice(&texture_data))
    }

    fn create_metalness_texture() -> (TextureDef, TextureData) {
        let texture_def = TextureDef {
            extents: Extents3D {
                width: 4,
                height: 4,
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::R8G8B8A8_UNORM,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Linear,
        };

        let mut texture_data = Vec::<u8>::with_capacity(64);
        for _index in 0..16 {
            texture_data.push(0);
            texture_data.push(0);
            texture_data.push(0);
            texture_data.push(255);
        }

        (texture_def, TextureData::from_slice(&texture_data))
    }

    fn create_roughness_texture() -> (TextureDef, TextureData) {
        let texture_def = TextureDef {
            extents: Extents3D {
                width: 4,
                height: 4,
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::R8G8B8A8_UNORM,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Linear,
        };

        let mut texture_data = Vec::<u8>::with_capacity(64);
        for _index in 0..16 {
            texture_data.push(0);
            texture_data.push(0);
            texture_data.push(0);
            texture_data.push(255);
        }

        (texture_def, TextureData::from_slice(&texture_data))
    }
}
