use lgn_ecs::prelude::Component;
use lgn_graphics_api::{
    BufferDef, CmdCopyBufferToTextureParams, DeviceContext, Extents3D, Format, MemoryAllocation,
    MemoryAllocationDef, MemoryUsage, ResourceFlags, ResourceState, ResourceUsage, Texture,
    TextureBarrier, TextureDef, TextureTiling,
};

use crate::{hl_gfx_api::HLCommandBuffer, resources::GpuUniformDataContext};

#[derive(Component)]
pub struct TextureComponent {
    texture_def: TextureDef,
    texture_data: Vec<Vec<u8>>,
    texture_id: u32,
}

impl TextureComponent {
    pub fn texture_id(&self) -> u32 {
        self.texture_id
    }

    pub fn texture_def(&self) -> &TextureDef {
        &self.texture_def
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        texture_data: Vec<Vec<u8>>,
        format: Format,
        width: u32,
        height: u32,
        mip_count: u32,
        data_context: &mut GpuUniformDataContext<'_>,
    ) -> Self {
        let texture_def = TextureDef {
            extents: Extents3D {
                width,
                height,
                depth: 1,
            },
            array_length: 1,
            mip_count,
            format,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_TRANSFERABLE,
            resource_flags: ResourceFlags::empty(),
            mem_usage: MemoryUsage::GpuOnly,
            tiling: TextureTiling::Optimal,
        };

        let gpu_index = data_context.aquire_gpu_texture_id();

        Self {
            texture_def,
            texture_data,
            texture_id: gpu_index,
        }
    }

    pub(crate) fn upload_texture(
        &mut self,
        device_context: &DeviceContext,
        cmd_buffer: &HLCommandBuffer<'_>,
        texture: &Texture,
    ) {
        assert!(self.texture_data.len() == self.texture_def.mip_count as usize);

        if !self.texture_data.is_empty() {
            for mip_level in 0..self.texture_def.mip_count {
                self.upload_texture_mip(device_context, cmd_buffer, texture, mip_level as u8);
            }
        }
        self.texture_data.clear();
    }

    fn upload_texture_mip(
        &self,
        device_context: &DeviceContext,
        cmd_buffer: &HLCommandBuffer<'_>,
        texture: &Texture,
        mip_level: u8,
    ) {
        upload_texture_data(
            device_context,
            cmd_buffer,
            texture,
            &self.texture_data[mip_level as usize],
            mip_level,
        );
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
