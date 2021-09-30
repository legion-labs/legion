// Don't use standard formatting in this file
#![allow(unused_attributes)]
#![allow(unused_variables)]
#![allow(clippy::unimplemented)]

use crate::prelude::*;
use raw_window_handle::HasRawWindowHandle;

//
// Root of the API
//
pub struct NullApi;

impl GfxApi for NullApi {
    fn device_context(&self) -> &NullDeviceContext {
        unimplemented!()
    }

    fn destroy(&mut self) -> GfxResult<()> {
        unimplemented!()
    }

    type DeviceContext = NullDeviceContext;
    type Buffer = NullBuffer;
    type Texture = NullTexture;
    type Sampler = NullSampler;
    type ShaderModule = NullShaderModule;
    type Shader = NullShader;
    type RootSignature = NullRootSignature;
    type Pipeline = NullPipeline;
    type DescriptorSetHandle = NullDescriptorSetHandle;
    type DescriptorSetArray = NullDescriptorSetArray;
    type Queue = NullQueue;
    type CommandPool = NullCommandPool;
    type CommandBuffer = NullCommandBuffer;
    type Fence = NullFence;
    type Semaphore = NullSemaphore;
    type Swapchain = NullSwapchain;
}

#[derive(Clone)]
pub struct NullDeviceContext;
impl DeviceContext<NullApi> for NullDeviceContext {
    fn device_info(&self) -> &DeviceInfo {
        unimplemented!()
    }

    fn create_queue(&self, queue_type: QueueType) -> GfxResult<NullQueue> {
        unimplemented!();
    }
    fn create_fence(&self) -> GfxResult<NullFence> {
        unimplemented!();
    }
    fn create_semaphore(&self) -> GfxResult<NullSemaphore> {
        unimplemented!();
    }
    fn create_swapchain(
        &self,
        raw_window_handle: &dyn HasRawWindowHandle,
        swapchain_def: &SwapchainDef,
    ) -> GfxResult<NullSwapchain> {
        unimplemented!();
    }
    fn create_sampler(&self, sampler_def: &SamplerDef) -> GfxResult<NullSampler> {
        unimplemented!();
    }
    fn create_texture(&self, texture_def: &TextureDef) -> GfxResult<NullTexture> {
        unimplemented!();
    }
    fn create_buffer(&self, buffer_def: &BufferDef) -> GfxResult<NullBuffer> {
        unimplemented!();
    }
    fn create_shader(&self, stages: Vec<ShaderStageDef<NullApi>>) -> GfxResult<NullShader> {
        unimplemented!();
    }
    fn create_root_signature(
        &self,
        root_signature_def: &RootSignatureDef<'_, NullApi>,
    ) -> GfxResult<NullRootSignature> {
        unimplemented!();
    }
    fn create_descriptor_set_array(
        &self,
        descriptor_set_array_def: &DescriptorSetArrayDef<'_, NullApi>,
    ) -> GfxResult<NullDescriptorSetArray> {
        unimplemented!();
    }
    fn create_graphics_pipeline(
        &self,
        graphics_pipeline_def: &GraphicsPipelineDef<'_, NullApi>,
    ) -> GfxResult<NullPipeline> {
        unimplemented!();
    }
    fn create_compute_pipeline(
        &self,
        compute_pipeline_def: &ComputePipelineDef<'_, NullApi>,
    ) -> GfxResult<NullPipeline> {
        unimplemented!();
    }
    fn create_shader_module(&self, data: ShaderModuleDef<'_>) -> GfxResult<NullShaderModule> {
        unimplemented!();
    }

    fn wait_for_fences(&self, fences: &[&NullFence]) -> GfxResult<()> {
        unimplemented!();
    }

    fn find_supported_format(
        &self,
        candidates: &[Format],
        resource_type: ResourceType,
    ) -> Option<Format> {
        unimplemented!();
    }
    fn find_supported_sample_count(&self, candidates: &[SampleCount]) -> Option<SampleCount> {
        unimplemented!();
    }
}

//
// Resources (Buffers, Textures, Samplers)
//
#[derive(Debug)]
pub struct NullBuffer;
impl Buffer<NullApi> for NullBuffer {
    fn buffer_def(&self) -> &BufferDef {
        unimplemented!()
    }
    fn map_buffer(&self) -> GfxResult<*mut u8> {
        unimplemented!()
    }
    fn unmap_buffer(&self) -> GfxResult<()> {
        unimplemented!()
    }
    fn mapped_memory(&self) -> Option<*mut u8> {
        unimplemented!()
    }
    fn copy_to_host_visible_buffer<T: Copy>(&self, data: &[T]) -> GfxResult<()> {
        unimplemented!()
    }
    fn copy_to_host_visible_buffer_with_offset<T: Copy>(
        &self,
        data: &[T],
        buffer_byte_offset: u64,
    ) -> GfxResult<()> {
        unimplemented!()
    }
}

#[derive(Clone, Debug)]
pub struct NullTexture;
impl Texture<NullApi> for NullTexture {
    fn texture_def(&self) -> &TextureDef {
        unimplemented!()
    }
    fn map_texture(&self) -> GfxResult<TextureSubResource<'_>> {
        unimplemented!()
    }
    fn unmap_texture(&self) -> GfxResult<()> {
        unimplemented!()
    }
}

#[derive(Clone, Debug)]
pub struct NullSampler;
impl Sampler<NullApi> for NullSampler {}

//
// Shaders/Pipelines
//
#[derive(Clone, Debug)]
pub struct NullShaderModule;
impl ShaderModule<NullApi> for NullShaderModule {}

#[derive(Clone, Debug)]
pub struct NullShader;
impl Shader<NullApi> for NullShader {
    fn pipeline_reflection(&self) -> &PipelineReflection {
        unimplemented!()
    }
}

#[derive(Clone, Debug)]
pub struct NullRootSignature;
impl RootSignature<NullApi> for NullRootSignature {
    fn pipeline_type(&self) -> PipelineType {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct NullPipeline;
impl Pipeline<NullApi> for NullPipeline {
    fn pipeline_type(&self) -> PipelineType {
        unimplemented!();
    }
    fn root_signature(&self) -> &NullRootSignature {
        unimplemented!();
    }
}

//
// Descriptor Sets
//
#[derive(Clone, Debug)]
pub struct NullDescriptorSetHandle;
impl DescriptorSetHandle<NullApi> for NullDescriptorSetHandle {}

#[derive(Debug)]
pub struct NullDescriptorSetArray;
impl DescriptorSetArray<NullApi> for NullDescriptorSetArray {
    fn handle(&self, array_index: u32) -> Option<NullDescriptorSetHandle> {
        unimplemented!();
    }
    fn root_signature(&self) -> &NullRootSignature {
        unimplemented!();
    }
    fn update_descriptor_set(&mut self, params: &[DescriptorUpdate<'_, NullApi>]) -> GfxResult<()> {
        unimplemented!();
    }
    fn queue_descriptor_set_update(
        &mut self,
        update: &DescriptorUpdate<'_, NullApi>,
    ) -> GfxResult<()> {
        unimplemented!();
    }
    fn flush_descriptor_set_updates(&mut self) -> GfxResult<()> {
        unimplemented!();
    }
}

//
// Queues, Command Buffers
//
#[derive(Clone, Debug)]
pub struct NullQueue;
impl Queue<NullApi> for NullQueue {
    fn device_context(&self) -> &NullDeviceContext {
        unimplemented!()
    }
    fn queue_id(&self) -> u32 {
        unimplemented!();
    }
    fn queue_type(&self) -> QueueType {
        unimplemented!();
    }
    fn create_command_pool(&self, command_pool_def: &CommandPoolDef) -> GfxResult<NullCommandPool> {
        unimplemented!();
    }
    fn submit(
        &self,
        command_buffers: &[&NullCommandBuffer],
        wait_semaphores: &[&NullSemaphore],
        signal_semaphores: &[&NullSemaphore],
        signal_fence: Option<&NullFence>,
    ) -> GfxResult<()> {
        unimplemented!();
    }
    fn present(
        &self,
        swapchain: &NullSwapchain,
        wait_semaphores: &[&NullSemaphore],
        image_index: u32,
    ) -> GfxResult<PresentSuccessResult> {
        unimplemented!()
    }
    fn wait_for_queue_idle(&self) -> GfxResult<()> {
        unimplemented!()
    }
}

pub struct NullCommandPool;
impl CommandPool<NullApi> for NullCommandPool {
    fn device_context(&self) -> &NullDeviceContext {
        unimplemented!()
    }
    fn create_command_buffer(
        &self,
        command_buffer_def: &CommandBufferDef,
    ) -> GfxResult<NullCommandBuffer> {
        unimplemented!()
    }
    fn reset_command_pool(&self) -> GfxResult<()> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct NullCommandBuffer;
impl CommandBuffer<NullApi> for NullCommandBuffer {
    fn begin(&self) -> GfxResult<()> {
        unimplemented!()
    }
    fn end(&self) -> GfxResult<()> {
        unimplemented!()
    }
    fn return_to_pool(&self) -> GfxResult<()> {
        unimplemented!()
    }

    fn cmd_begin_render_pass(
        &self,
        color_targets: &[ColorRenderTargetBinding<'_, NullApi>],
        depth_target: Option<DepthStencilRenderTargetBinding<'_, NullApi>>,
    ) -> GfxResult<()> {
        unimplemented!()
    }
    fn cmd_end_render_pass(&self) -> GfxResult<()> {
        unimplemented!()
    }

    fn cmd_set_viewport(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        depth_min: f32,
        depth_max: f32,
    ) -> GfxResult<()> {
        unimplemented!()
    }
    fn cmd_set_scissor(&self, x: u32, y: u32, width: u32, height: u32) -> GfxResult<()> {
        unimplemented!()
    }
    fn cmd_set_stencil_reference_value(&self, value: u32) -> GfxResult<()> {
        unimplemented!()
    }

    fn cmd_bind_pipeline(&self, pipeline: &NullPipeline) -> GfxResult<()> {
        unimplemented!()
    }
    fn cmd_bind_vertex_buffers(
        &self,
        first_binding: u32,
        bindings: &[VertexBufferBinding<'_, NullApi>],
    ) -> GfxResult<()> {
        unimplemented!()
    }
    fn cmd_bind_index_buffer(&self, binding: &IndexBufferBinding<'_, NullApi>) -> GfxResult<()> {
        unimplemented!()
    }
    fn cmd_bind_descriptor_set(
        &self,
        descriptor_set_array: &NullDescriptorSetArray,
        index: u32,
    ) -> GfxResult<()> {
        unimplemented!()
    }
    fn cmd_bind_descriptor_set_handle(
        &self,
        root_signature: &NullRootSignature,
        set_index: u32,
        descriptor_set_handle: &NullDescriptorSetHandle,
    ) -> GfxResult<()> {
        unimplemented!()
    }

    fn cmd_draw(&self, vertex_count: u32, first_vertex: u32) -> GfxResult<()> {
        unimplemented!()
    }
    fn cmd_draw_instanced(
        &self,
        vertex_count: u32,
        first_vertex: u32,
        instance_count: u32,
        first_instance: u32,
    ) -> GfxResult<()> {
        unimplemented!()
    }
    fn cmd_draw_indexed(
        &self,
        index_count: u32,
        first_index: u32,
        vertex_offset: i32,
    ) -> GfxResult<()> {
        unimplemented!()
    }
    fn cmd_draw_indexed_instanced(
        &self,
        index_count: u32,
        first_index: u32,
        instance_count: u32,
        first_instance: u32,
        vertex_offset: i32,
    ) -> GfxResult<()> {
        unimplemented!()
    }

    fn cmd_dispatch(
        &self,
        group_count_x: u32,
        group_count_y: u32,
        group_count_z: u32,
    ) -> GfxResult<()> {
        unimplemented!()
    }

    fn cmd_resource_barrier(
        &self,
        buffer_barriers: &[BufferBarrier<'_, NullApi>],
        texture_barriers: &[TextureBarrier<'_, NullApi>],
    ) -> GfxResult<()> {
        unimplemented!()
    }
    fn cmd_copy_buffer_to_buffer(
        &self,
        src_buffer: &NullBuffer,
        dst_buffer: &NullBuffer,
        src_offset: u64,
        dst_offset: u64,
        size: u64,
    ) -> GfxResult<()> {
        unimplemented!()
    }
    fn cmd_copy_buffer_to_texture(
        &self,
        src_buffer: &NullBuffer,
        dst_texture: &NullTexture,
        params: &CmdCopyBufferToTextureParams,
    ) -> GfxResult<()> {
        unimplemented!()
    }

    fn cmd_blit_texture(
        &self,
        src_texture: &NullTexture,
        dst_texture: &NullTexture,
        params: &CmdBlitParams,
    ) -> GfxResult<()> {
        unimplemented!()
    }

    fn cmd_copy_image(
        &self,
        src_texture: &NullTexture,
        dst_texture: &NullTexture,
        params: &CmdCopyTextureParams,
    ) -> GfxResult<()> {
        unimplemented!()
    }
}

//
// Fences and Semaphores
//
pub struct NullFence;
impl Fence<NullApi> for NullFence {
    fn wait(&self) -> GfxResult<()> {
        unimplemented!();
    }
    fn wait_for_fences(device_context: &NullDeviceContext, fences: &[&Self]) -> GfxResult<()> {
        unimplemented!();
    }
    fn get_fence_status(&self) -> GfxResult<FenceStatus> {
        unimplemented!();
    }
}

pub struct NullSemaphore;
impl Semaphore<NullApi> for NullSemaphore {}

//
// Swapchain
//
pub struct NullSwapchain;
impl Swapchain<NullApi> for NullSwapchain {
    fn swapchain_def(&self) -> &SwapchainDef {
        unimplemented!()
    }
    fn image_count(&self) -> usize {
        unimplemented!()
    }
    fn format(&self) -> Format {
        unimplemented!()
    }
    fn acquire_next_image_fence(
        &mut self,
        fence: &NullFence,
    ) -> GfxResult<SwapchainImage<NullApi>> {
        unimplemented!()
    }
    fn acquire_next_image_semaphore(
        &mut self,
        semaphore: &NullSemaphore,
    ) -> GfxResult<SwapchainImage<NullApi>> {
        unimplemented!()
    }
    fn rebuild(&mut self, swapchain_def: &SwapchainDef) -> GfxResult<()> {
        unimplemented!()
    }
}
