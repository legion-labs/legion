use lgn_graphics_api::prelude::*;

use crate::{
    components::TextureData,
    resources::{TransientBufferAllocator, TransientCommandBufferAllocator},
    GraphicsQueue,
};

use super::{RenderCommand, RenderResources};

pub struct UploadGPUBuffer {
    pub src_data: Vec<u8>,
    pub dst_buffer: Buffer,
    pub dst_offset: u64,
}

pub struct UploadGPUTexture {
    pub src_data: TextureData,
    pub dst_texture: Texture,
}

pub enum UploadGPUResource {
    Buffer(UploadGPUBuffer),
    Texture(UploadGPUTexture),
}

pub struct UploadBufferCommand {
    pub src_buffer: Vec<u8>,
    pub dst_buffer: Buffer,
    pub dst_offset: u64,
}

impl RenderCommand<RenderResources> for UploadBufferCommand {
    fn execute(self, render_resources: &super::RenderResources) {
        let mut mng = render_resources.get_mut::<GpuUploadManager>();
        mng.push(UploadGPUResource::Buffer(UploadGPUBuffer {
            src_data: self.src_buffer,
            dst_buffer: self.dst_buffer,
            dst_offset: self.dst_offset,
        }));
    }
}

pub struct UploadTextureCommand {
    pub src_data: TextureData,
    pub dst_texture: Texture,
}

impl RenderCommand<RenderResources> for UploadTextureCommand {
    fn execute(self, render_resources: &super::RenderResources) {
        let mut mng = render_resources.get_mut::<GpuUploadManager>();
        mng.push(UploadGPUResource::Texture(UploadGPUTexture {
            src_data: self.src_data,
            dst_texture: self.dst_texture,
        }));
    }
}

pub struct GpuUploadManager {
    updates: Vec<UploadGPUResource>,
}

impl GpuUploadManager {
    pub fn new() -> Self {
        Self {
            updates: Vec::new(),
        }
    }

    pub fn push(&mut self, update: UploadGPUResource) {
        self.updates.push(update);
    }

    pub fn upload(
        &mut self,
        transient_commandbuffer_allocator: &mut TransientCommandBufferAllocator,
        transient_buffer_allocator: &mut TransientBufferAllocator,
        graphics_queue: &GraphicsQueue,
    ) {
        if self.updates.is_empty() {
            return;
        }

        let mut cmd_buffer_handle = transient_commandbuffer_allocator.acquire();
        let cmd_buffer = cmd_buffer_handle.as_mut();

        cmd_buffer.begin();

        for update in self.updates.drain(..) {
            match update {
                UploadGPUResource::Buffer(upload_buf) => {
                    let transient_alloc = transient_buffer_allocator
                        .copy_data_slice(&upload_buf.src_data, ResourceUsage::empty());

                    cmd_buffer.cmd_resource_barrier(
                        &[BufferBarrier {
                            buffer: &upload_buf.dst_buffer,
                            src_state: ResourceState::SHADER_RESOURCE,
                            dst_state: ResourceState::COPY_DST,
                            queue_transition: BarrierQueueTransition::None,
                        }],
                        &[],
                    );

                    cmd_buffer.cmd_copy_buffer_to_buffer(
                        transient_alloc.buffer(),
                        &upload_buf.dst_buffer,
                        &[BufferCopy {
                            src_offset: transient_alloc.byte_offset(),
                            dst_offset: upload_buf.dst_offset,
                            size: upload_buf.src_data.len() as u64,
                        }],
                    );

                    cmd_buffer.cmd_resource_barrier(
                        &[BufferBarrier {
                            buffer: &upload_buf.dst_buffer,
                            src_state: ResourceState::COPY_DST,
                            dst_state: ResourceState::SHADER_RESOURCE,
                            queue_transition: BarrierQueueTransition::None,
                        }],
                        &[],
                    );
                }
                UploadGPUResource::Texture(upload_tex) => {
                    let texture = upload_tex.dst_texture;
                    let texture_data = upload_tex.src_data;
                    let mip_slices = texture_data.data();

                    cmd_buffer.cmd_resource_barrier(
                        &[],
                        &[TextureBarrier::state_transition(
                            &texture,
                            ResourceState::UNDEFINED,
                            ResourceState::COPY_DST,
                        )],
                    );

                    for (mip_level, mip_data) in mip_slices.iter().enumerate() {
                        let transient_alloc = transient_buffer_allocator
                            .copy_data_slice(mip_data, ResourceUsage::empty());

                        cmd_buffer.cmd_copy_buffer_to_texture(
                            transient_alloc.buffer(),
                            &texture,
                            &CmdCopyBufferToTextureParams {
                                buffer_offset: transient_alloc.byte_offset(),
                                array_layer: 0,
                                mip_level: mip_level as u8,
                            },
                        );
                    }

                    cmd_buffer.cmd_resource_barrier(
                        &[],
                        &[TextureBarrier::state_transition(
                            &texture,
                            ResourceState::COPY_DST,
                            ResourceState::SHADER_RESOURCE,
                        )],
                    );
                }
            }
        }

        cmd_buffer.end();

        graphics_queue
            .queue_mut()
            .submit(&[cmd_buffer], &[], &[], None);

        transient_commandbuffer_allocator.release(cmd_buffer_handle);
    }
}
