use std::collections::BTreeMap;

use lgn_data_runtime::ResourceTypeAndId;
use lgn_graphics_api::{
    BufferDef, CmdCopyBufferToTextureParams, DeviceContext, Extents3D, Format, MemoryAllocation,
    MemoryAllocationDef, MemoryUsage, ResourceFlags, ResourceState, ResourceUsage, Texture,
    TextureBarrier, TextureDef, TextureTiling, TextureView, TextureViewDef,
};
use lgn_graphics_data::{runtime_texture::TextureReferenceType, TextureFormat};
use lgn_tracing::span_fn;

use crate::{components::TextureComponent, hl_gfx_api::HLCommandBuffer};

use super::{IndexAllocator, IndexBlock};

struct UploadTextureJobs {
    texture: Texture,
    texture_data: Vec<Vec<u8>>,
}

pub struct BindlessTextureManager {
    textures: Vec<Texture>,
    bindless_array: Vec<TextureView>,
    upload_jobs: Vec<UploadTextureJobs>,
    index_allocator: IndexAllocator,
    ref_to_gpu_id: BTreeMap<ResourceTypeAndId, u32>,
    default_texture_id: u32,
}

impl BindlessTextureManager {
    pub fn new(device_context: &DeviceContext, array_size: usize) -> Self {
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

        let index_allocator = IndexAllocator::new(256);

        let mut index_block = None;
        let default_texture_id = index_allocator.acquire_index(&mut index_block);
        index_allocator.release_index_block(index_block.unwrap());

        let default_black_texture = device_context.create_texture(&texture_def);

        let default_black_texture_view_def = TextureViewDef::as_shader_resource_view(&texture_def);

        let descriptor_array =
            vec![default_black_texture.create_view(&default_black_texture_view_def,); array_size];

        let mut texture_data = Vec::<u8>::with_capacity(64);
        for _index in 0..16 {
            texture_data.push(0);
            texture_data.push(0);
            texture_data.push(0);
            texture_data.push(255);
        }

        let upload_default = UploadTextureJobs {
            texture: default_black_texture.clone(),
            texture_data: vec![texture_data],
        };

        Self {
            textures: vec![default_black_texture],
            bindless_array: descriptor_array,
            upload_jobs: vec![upload_default],
            index_allocator,
            ref_to_gpu_id: BTreeMap::new(),
            default_texture_id,
        }
    }

    pub fn allocate_texture(
        &mut self,
        device_context: &DeviceContext,
        texture_component: &mut TextureComponent,
        index_block: &mut Option<IndexBlock>,
    ) {
        let bindless_id = self.index_allocator.acquire_index(index_block);

        if let Some(stored_id) = self.ref_to_gpu_id.get_mut(&texture_component.texture_id) {
            *stored_id = bindless_id;
        } else {
            self.ref_to_gpu_id
                .insert(texture_component.texture_id, bindless_id);
        }

        let format = match texture_component.format {
            TextureFormat::BC1 => Format::BC1_RGBA_UNORM_BLOCK,
            TextureFormat::BC3 => Format::BC3_UNORM_BLOCK,
            TextureFormat::BC4 => Format::BC4_UNORM_BLOCK,
            TextureFormat::BC7 => Format::BC7_UNORM_BLOCK,
        };

        let texture_def = TextureDef {
            extents: Extents3D {
                width: texture_component.width,
                height: texture_component.height,
                depth: 1,
            },
            array_length: 1,
            mip_count: texture_component.texture_data.len() as u32,
            format,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Optimal,
        };

        let new_texture = device_context.create_texture(&texture_def);

        let texture_view_def = TextureViewDef::as_shader_resource_view(&texture_def);

        self.bindless_array[bindless_id as usize] = new_texture.create_view(&texture_view_def);
        self.textures.push(new_texture.clone());

        self.upload_jobs.push(UploadTextureJobs {
            texture: new_texture,
            texture_data: std::mem::take(&mut texture_component.texture_data),
        });
    }

    #[span_fn]
    #[allow(clippy::needless_pass_by_value)]
    pub fn upload_textures(
        &mut self,
        device_context: &DeviceContext,
        cmd_buffer: &HLCommandBuffer<'_>,
    ) {
        for upload in self.upload_jobs.drain(..) {
            for mip_level in 0..upload.texture_data.len() as u8 {
                upload_texture_data(
                    device_context,
                    cmd_buffer,
                    &upload.texture,
                    &upload.texture_data[mip_level as usize],
                    mip_level,
                );
            }
        }
    }

    pub fn return_index_block(&self, index_block: Option<IndexBlock>) {
        if let Some(block) = index_block {
            self.index_allocator.release_index_block(block);
        }
    }

    pub fn bindless_id_for_texture(&self, optional_id: &Option<TextureReferenceType>) -> u32 {
        if let Some(texture_id) = optional_id {
            if let Some(id) = self.ref_to_gpu_id.get(&texture_id.id()) {
                *id
            } else {
                self.default_texture_id
            }
        } else {
            u32::MAX
        }
    }

    pub fn default_black_texture_view(&self) -> TextureView {
        self.bindless_array[0].clone()
    }

    pub fn bindless_texures_for_update(&self) -> Vec<TextureView> {
        self.bindless_array.clone()
    }
}

pub fn upload_texture_data(
    device_context: &DeviceContext,
    cmd_buffer: &HLCommandBuffer<'_>,
    texture: &Texture,
    data: &[u8],
    mip_level: u8,
) {
    let staging_buffer = device_context.create_buffer(&BufferDef::for_staging_buffer_data(
        data,
        ResourceUsage::empty(),
    ));

    let alloc_def = MemoryAllocationDef {
        memory_usage: MemoryUsage::CpuToGpu,
        always_mapped: true,
    };

    let buffer_memory = MemoryAllocation::from_buffer(device_context, &staging_buffer, &alloc_def);

    buffer_memory.copy_to_host_visible_buffer(data);

    cmd_buffer.resource_barrier(
        &[],
        &[TextureBarrier::state_transition(
            texture,
            ResourceState::UNDEFINED,
            ResourceState::COPY_DST,
        )],
    );

    cmd_buffer.copy_buffer_to_texture(
        &staging_buffer,
        texture,
        &CmdCopyBufferToTextureParams {
            mip_level,
            ..CmdCopyBufferToTextureParams::default()
        },
    );

    cmd_buffer.resource_barrier(
        &[],
        &[TextureBarrier::state_transition(
            texture,
            ResourceState::COPY_DST,
            ResourceState::SHADER_RESOURCE,
        )],
    );
}
