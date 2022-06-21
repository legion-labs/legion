use lgn_graphics_data::Color;
use strum::IntoEnumIterator;

use lgn_graphics_api::{
    DeviceContext, Extents3D, Format, MemoryUsage, ResourceFlags, ResourceUsage, Sampler,
    SamplerDef, TextureDef, TextureTiling, TextureView, TextureViewDef,
};

use crate::core::{RenderCommandBuilder, UploadTextureCommand};

use super::{PersistentDescriptorSetManager, SamplerSlot, TextureData, TextureSlot};

#[derive(Clone, Copy, strum::EnumCount, strum::EnumIter)]
pub enum SharedTextureId {
    Albedo,
    Normal,
    Metalness,
    Roughness,
}

#[derive(Clone)]
struct DefaultSampler {
    _sampler: Sampler,
    bindless_slot: SamplerSlot,
}

#[derive(Clone)]
struct DefaultTexture {
    _texture_view: TextureView,
    bindless_slot: TextureSlot,
}

pub struct SharedResourcesManager {
    default_sampler: DefaultSampler,
    default_textures: Vec<DefaultTexture>,
}

impl SharedResourcesManager {
    pub fn new(
        render_commands: &mut RenderCommandBuilder,
        device_context: &DeviceContext,
        persistent_descriptor_set_manager: &mut PersistentDescriptorSetManager,
    ) -> Self {
        let default_sampler =
            Self::create_default_sampler(device_context, persistent_descriptor_set_manager);

        let default_textures = Self::create_default_textures(
            render_commands,
            device_context,
            persistent_descriptor_set_manager,
        );

        Self {
            default_sampler,
            default_textures,
        }
    }

    pub fn default_texture_slot(&self, shared_texture_id: SharedTextureId) -> TextureSlot {
        self.default_textures[shared_texture_id as usize].bindless_slot
    }

    pub fn default_sampler_slot(&self) -> SamplerSlot {
        self.default_sampler.bindless_slot
    }

    fn create_texture(
        render_commands: &mut RenderCommandBuilder,
        device_context: &DeviceContext,
        shared_texture_id: SharedTextureId,
    ) -> TextureView {
        let (texture_def, texture_data, name) = match shared_texture_id {
            SharedTextureId::Albedo => Self::create_albedo_texture(),
            SharedTextureId::Normal => Self::create_normal_texture(),
            SharedTextureId::Metalness => Self::create_metalness_texture(),
            SharedTextureId::Roughness => Self::create_roughness_texture(),
        };

        let texture = device_context.create_texture(texture_def, &name);
        let texture_view = texture.create_view(TextureViewDef::as_shader_resource_view(
            texture.definition(),
        ));

        render_commands.push(UploadTextureCommand {
            src_data: texture_data,
            dst_texture: texture,
        });

        texture_view
    }

    fn create_albedo_texture() -> (TextureDef, TextureData, String) {
        let texture_def = TextureDef {
            extents: Extents3D {
                width: 2,
                height: 2,
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::R8G8B8A8_SRGB,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            memory_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Linear,
        };

        let mut texture_data = [Color::default(); 4];

        // https://colorpicker.me/#9b0eab
        texture_data[0] = Color::new(155, 14, 171, 255);
        texture_data[1] = Color::new(155, 14, 171, 255);
        texture_data[2] = Color::new(155, 14, 171, 255);
        texture_data[3] = Color::new(155, 14, 171, 255);

        (
            texture_def,
            TextureData::from_slice(&texture_data),
            "default_albedo".to_string(),
        )
    }

    fn create_normal_texture() -> (TextureDef, TextureData, String) {
        let texture_def = TextureDef {
            extents: Extents3D {
                width: 2,
                height: 2,
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::R8G8B8A8_UNORM,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            memory_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Linear,
        };

        let mut texture_data = [Color::default(); 4];

        texture_data[0] = Color::new(127, 127, 255, 255);
        texture_data[1] = Color::new(127, 127, 255, 255);
        texture_data[2] = Color::new(127, 127, 255, 255);
        texture_data[3] = Color::new(127, 127, 255, 255);

        (
            texture_def,
            TextureData::from_slice(&texture_data),
            "default_normal".to_string(),
        )
    }

    fn create_metalness_texture() -> (TextureDef, TextureData, String) {
        let texture_def = TextureDef {
            extents: Extents3D {
                width: 2,
                height: 2,
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::R8_UNORM,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            memory_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Linear,
        };

        let mut texture_data = [0_u8; 4];

        texture_data[0] = 0;
        texture_data[1] = 0;
        texture_data[2] = 0;
        texture_data[3] = 0;

        (
            texture_def,
            TextureData::from_slice(&texture_data),
            "Metalness".to_string(),
        )
    }

    fn create_roughness_texture() -> (TextureDef, TextureData, String) {
        let texture_def = TextureDef {
            extents: Extents3D {
                width: 2,
                height: 2,
                depth: 1,
            },
            array_length: 1,
            mip_count: 1,
            format: Format::R8_UNORM,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            memory_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Linear,
        };

        let mut texture_data = [0_u8; 4];

        texture_data[0] = 240;
        texture_data[1] = 240;
        texture_data[2] = 240;
        texture_data[3] = 240;

        (
            texture_def,
            TextureData::from_slice(&texture_data),
            "Roughness".to_string(),
        )
    }

    fn create_default_sampler(
        device_context: &DeviceContext,
        persistent_descriptor_set_manager: &mut PersistentDescriptorSetManager,
    ) -> DefaultSampler {
        let sampler = device_context.create_sampler(SamplerDef::default());
        let bindless_slot = persistent_descriptor_set_manager.allocate_sampler_slot(&sampler);
        DefaultSampler {
            _sampler: sampler,
            bindless_slot,
        }
    }

    fn create_default_textures(
        render_commands: &mut RenderCommandBuilder,
        device_context: &DeviceContext,
        persistent_descriptor_set_manager: &mut PersistentDescriptorSetManager,
    ) -> Vec<DefaultTexture> {
        SharedTextureId::iter()
            .map(|shared_texture_id| {
                let texture_view =
                    Self::create_texture(render_commands, device_context, shared_texture_id);
                DefaultTexture {
                    _texture_view: texture_view.clone(),
                    bindless_slot: persistent_descriptor_set_manager
                        .allocate_texture_slot(&texture_view),
                }
            })
            .collect::<Vec<_>>()
    }
}
