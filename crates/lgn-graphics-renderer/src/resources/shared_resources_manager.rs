use lgn_graphics_data::Color;
use strum::{EnumCount, IntoEnumIterator};

use lgn_graphics_api::{
    BufferDef, CmdCopyBufferToTextureParams, CommandBufferDef, CommandPoolDef, Extents3D, Format,
    MemoryAllocation, MemoryAllocationDef, MemoryUsage, QueueType, ResourceFlags, ResourceState,
    ResourceUsage, TextureBarrier, TextureDef, TextureTiling, TextureView, TextureViewDef,
};

use crate::{components::TextureData, Renderer};

use super::PersistentDescriptorSetManager;

#[derive(Clone, Copy, strum::EnumCount, strum::EnumIter)]
pub enum SharedTextureId {
    Albedo,
    Normal,
    Metalness,
    Roughness,
}

#[derive(Debug, Clone)]
struct SharedTexture {
    _texture_view: TextureView,
    bindless_index: u32,
}

pub struct SharedResourcesManager {
    textures: [SharedTexture; SharedTextureId::COUNT],
}

impl SharedResourcesManager {
    pub fn new(
        renderer: &Renderer,
        persistent_descriptor_set_manager: &mut PersistentDescriptorSetManager,
    ) -> Self {
        let shared_textures =
            Self::create_shared_textures(renderer, persistent_descriptor_set_manager);

        Self {
            textures: shared_textures.try_into().unwrap(),
        }
    }

    pub fn default_texture_bindless_index(&self, shared_texture_id: SharedTextureId) -> u32 {
        self.textures[shared_texture_id as usize].bindless_index
    }

    fn create_texture(renderer: &Renderer, shared_texture_id: SharedTextureId) -> TextureView {
        let (texture_def, texture_data) = match shared_texture_id {
            SharedTextureId::Albedo => Self::create_albedo_texture(),
            SharedTextureId::Normal => Self::create_normal_texture(),
            SharedTextureId::Metalness => Self::create_metalness_texture(),
            SharedTextureId::Roughness => Self::create_roughness_texture(),
        };

        let device_context = renderer.device_context();
        let texture = device_context.create_texture(&texture_def);
        let texture_view =
            texture.create_view(&TextureViewDef::as_shader_resource_view(&texture_def));

        let graphics_queue = renderer.graphics_queue_guard(QueueType::Graphics);

        let cmd_buffer_pool = graphics_queue
            .create_command_pool(&CommandPoolDef { transient: false })
            .unwrap();

        let cmd_buffer = cmd_buffer_pool
            .create_command_buffer(&CommandBufferDef {
                is_secondary: false,
            })
            .unwrap();
        cmd_buffer.begin().unwrap();

        {
            let data = texture_data.data()[0].as_slice();

            // todo: this code must be completly rewritten (-> upload manager)
            let staging_buffer = device_context.create_buffer(&BufferDef::for_staging_buffer_data(
                data,
                ResourceUsage::empty(),
            ));

            let alloc_def = MemoryAllocationDef {
                memory_usage: MemoryUsage::CpuToGpu,
                always_mapped: true,
            };

            let buffer_memory =
                MemoryAllocation::from_buffer(device_context, &staging_buffer, &alloc_def);

            buffer_memory.copy_to_host_visible_buffer(data);

            // todo: not needed here
            cmd_buffer.cmd_resource_barrier(
                &[],
                &[TextureBarrier::state_transition(
                    &texture,
                    ResourceState::UNDEFINED,
                    ResourceState::COPY_DST,
                )],
            );

            cmd_buffer.cmd_copy_buffer_to_texture(
                &staging_buffer,
                &texture,
                &CmdCopyBufferToTextureParams {
                    buffer_offset: 0,
                    array_layer: 0,
                    mip_level: 0,
                },
            );

            // todo: not needed here
            cmd_buffer.cmd_resource_barrier(
                &[],
                &[TextureBarrier::state_transition(
                    &texture,
                    ResourceState::COPY_DST,
                    ResourceState::SHADER_RESOURCE,
                )],
            );
        }

        cmd_buffer.end().unwrap();

        graphics_queue
            .submit(&[&cmd_buffer], &[], &[], None)
            .unwrap();

        graphics_queue.wait_for_queue_idle().unwrap();

        texture_view
    }

    fn create_albedo_texture() -> (TextureDef, TextureData) {
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
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Linear,
        };

        let mut texture_data = [Color::default(); 4];

        // https://colorpicker.me/#9b0eab
        texture_data[0] = Color::new(155, 14, 171, 255);
        texture_data[1] = Color::new(155, 14, 171, 255);
        texture_data[2] = Color::new(155, 14, 171, 255);
        texture_data[3] = Color::new(155, 14, 171, 255);

        (texture_def, TextureData::from_slice(&texture_data))
    }

    fn create_normal_texture() -> (TextureDef, TextureData) {
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
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Linear,
        };

        let mut texture_data = [Color::default(); 4];

        texture_data[0] = Color::new(0, 0, 127, 255);
        texture_data[1] = Color::new(0, 0, 127, 255);
        texture_data[2] = Color::new(0, 0, 127, 255);
        texture_data[3] = Color::new(0, 0, 127, 255);

        (texture_def, TextureData::from_slice(&texture_data))
    }

    fn create_metalness_texture() -> (TextureDef, TextureData) {
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
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Linear,
        };

        let mut texture_data = [0_u8; 4];

        texture_data[0] = 0;
        texture_data[1] = 0;
        texture_data[2] = 0;
        texture_data[3] = 0;

        (texture_def, TextureData::from_slice(&texture_data))
    }

    fn create_roughness_texture() -> (TextureDef, TextureData) {
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
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Linear,
        };

        let mut texture_data = [0_u8; 4];

        texture_data[0] = 240;
        texture_data[1] = 240;
        texture_data[2] = 240;
        texture_data[3] = 240;

        (texture_def, TextureData::from_slice(&texture_data))
    }

    fn create_shared_textures(
        renderer: &Renderer,
        persistent_descriptor_set_manager: &mut PersistentDescriptorSetManager,
    ) -> Vec<SharedTexture> {
        SharedTextureId::iter()
            .map(|shared_texture_id| {
                let texture_view = Self::create_texture(renderer, shared_texture_id);
                SharedTexture {
                    _texture_view: texture_view.clone(),
                    bindless_index: persistent_descriptor_set_manager
                        .set_bindless_texture(&texture_view),
                }
            })
            .collect::<Vec<_>>()
    }
}
