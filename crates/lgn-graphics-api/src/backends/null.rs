#![cfg_attr(
    not(any(feature = "vulkan")),
    allow(
        dead_code,
        unused_attributes,
        unused_variables,
        clippy::needless_pass_by_value,
        clippy::unimplemented,
        clippy::unused_self
    )
)]
use std::marker::PhantomData;

use raw_window_handle::HasRawWindowHandle;

use crate::{
    ApiDef, Buffer, BufferBarrier, BufferCopy, BufferDef, BufferMappingInfo, CmdBlitParams,
    CmdCopyBufferToTextureParams, CmdCopyTextureParams, ColorRenderTargetBinding, CommandBuffer,
    CommandBufferDef, CommandPool, CommandPoolDef, ComputePipelineDef,
    DepthStencilRenderTargetBinding, Descriptor, DescriptorHeapDef, DescriptorHeapPartition,
    DescriptorRef, DescriptorSet, DescriptorSetHandle, DescriptorSetLayout, DescriptorSetWriter,
    DeviceContext, DeviceInfo, ExtensionMode, Fence, FenceStatus, Format, GfxResult,
    GraphicsPipelineDef, IndexBufferBinding, MemoryAllocation, MemoryAllocationDef,
    PagedBufferAllocation, Pipeline, PipelineType, PlaneSlice, PresentSuccessResult, Queue,
    QueueType, RootSignature, RootSignatureDef, SamplerDef, Semaphore, ShaderModuleDef, Swapchain,
    SwapchainDef, SwapchainImage, Texture, TextureBarrier, TextureDef, TextureSubResource,
    TextureViewDef, VertexBufferBinding,
};

////////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub(crate) struct NullApi;

impl NullApi {
    /// # Safety
    /// Null Api implementation, no safety measure needs to be implemented
    #[allow(unsafe_code)]
    pub unsafe fn new(_api_def: &ApiDef) -> GfxResult<(Self, DeviceContext)> {
        unimplemented!()
    }

    pub fn destroy(_device_context: &DeviceContext) {}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct NullInstance;

////////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub(crate) struct NullDeviceContext;

impl NullDeviceContext {
    pub(crate) fn new(
        _instance: &NullInstance,
        _windowing_mode: ExtensionMode,
        _video_mode: ExtensionMode,
    ) -> GfxResult<(Self, DeviceInfo)> {
        unimplemented!()
    }

    pub(crate) fn destroy(&mut self) {
        unimplemented!()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
///
#[derive(Debug)]
pub(crate) struct NullBuffer;

impl NullBuffer {
    pub fn new(device_context: &DeviceContext, buffer_def: &BufferDef) -> Self {
        unimplemented!()
    }

    pub(crate) fn destroy(&self, _device_context: &DeviceContext, _buffer_def: &BufferDef) {
        unimplemented!()
    }
}

impl Buffer {
    pub(crate) fn backend_required_alignment(&self) -> u64 {
        unimplemented!()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub(crate) struct NullCommandBuffer;

impl NullCommandBuffer {
    pub(crate) fn new(
        command_pool: &CommandPool,
        command_buffer_def: &CommandBufferDef,
    ) -> GfxResult<Self> {
        unimplemented!()
    }
}

impl CommandBuffer {
    pub(crate) fn backend_begin(&self) -> GfxResult<()> {
        unimplemented!()
    }

    pub(crate) fn backend_end(&self) -> GfxResult<()> {
        unimplemented!()
    }

    pub(crate) fn backend_cmd_begin_render_pass(
        &self,
        color_targets: &[ColorRenderTargetBinding<'_>],
        depth_target: &Option<DepthStencilRenderTargetBinding<'_>>,
    ) -> GfxResult<()> {
        unimplemented!()
    }

    pub(crate) fn backend_cmd_end_render_pass(&self) {
        unimplemented!()
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn backend_cmd_set_viewport(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        depth_min: f32,
        depth_max: f32,
    ) {
        unimplemented!()
    }

    pub(crate) fn backend_cmd_set_scissor(&self, x: u32, y: u32, width: u32, height: u32) {
        unimplemented!()
    }

    pub(crate) fn backend_cmd_set_stencil_reference_value(&self, value: u32) {
        unimplemented!()
    }

    pub(crate) fn backend_cmd_bind_pipeline(&self, pipeline: &Pipeline) {
        unimplemented!()
    }

    pub(crate) fn backend_cmd_bind_vertex_buffers(
        &self,
        first_binding: u32,
        bindings: &[VertexBufferBinding<'_>],
    ) {
        unimplemented!()
    }

    pub(crate) fn backend_cmd_bind_index_buffer(&self, binding: &IndexBufferBinding<'_>) {
        unimplemented!()
    }

    pub(crate) fn backend_cmd_bind_descriptor_set_handle(
        &self,
        pipeline_type: PipelineType,
        root_signature: &RootSignature,
        set_index: u32,
        descriptor_set_handle: DescriptorSetHandle,
    ) {
        unimplemented!()
    }

    pub(crate) fn backend_cmd_push_constant(&self, root_signature: &RootSignature, data: &[u8]) {
        unimplemented!()
    }

    pub(crate) fn backend_cmd_draw(&self, vertex_count: u32, first_vertex: u32) {
        unimplemented!()
    }

    pub(crate) fn backend_cmd_draw_instanced(
        &self,
        vertex_count: u32,
        first_vertex: u32,
        instance_count: u32,
        first_instance: u32,
    ) {
        unimplemented!()
    }

    pub(crate) fn backend_cmd_draw_indexed(
        &self,
        index_count: u32,
        first_index: u32,
        vertex_offset: i32,
    ) {
        unimplemented!()
    }

    pub(crate) fn backend_cmd_draw_indexed_instanced(
        &self,
        index_count: u32,
        first_index: u32,
        instance_count: u32,
        first_instance: u32,
        vertex_offset: i32,
    ) {
        unimplemented!()
    }

    pub(crate) fn backend_cmd_dispatch(
        &self,
        group_count_x: u32,
        group_count_y: u32,
        group_count_z: u32,
    ) {
        unimplemented!()
    }

    pub(crate) fn backedn_cmd_resource_barrier(
        &self,
        buffer_barriers: &[BufferBarrier<'_>],
        texture_barriers: &[TextureBarrier<'_>],
    ) {
        unimplemented!()
    }

    pub(crate) fn backend_cmd_fill_buffer(
        &self,
        dst_buffer: &Buffer,
        offset: u64,
        size: u64,
        data: u32,
    ) {
        unimplemented!()
    }

    pub(crate) fn backend_cmd_copy_buffer_to_buffer(
        &self,

        src_buffer: &Buffer,
        dst_buffer: &Buffer,
        copy_data: &[BufferCopy],
    ) {
        unimplemented!()
    }

    pub(crate) fn backend_cmd_copy_buffer_to_texture(
        &self,

        src_buffer: &Buffer,
        dst_texture: &Texture,
        params: &CmdCopyBufferToTextureParams,
    ) {
        unimplemented!()
    }

    pub(crate) fn backend_cmd_blit_texture(
        &self,

        src_texture: &Texture,
        dst_texture: &Texture,
        params: &CmdBlitParams,
    ) {
        unimplemented!()
    }

    pub(crate) fn backend_cmd_copy_image(
        &self,

        src_texture: &Texture,
        dst_texture: &Texture,
        params: &CmdCopyTextureParams,
    ) {
        unimplemented!()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub(crate) struct NullCommandPool;

impl NullCommandPool {
    pub(crate) fn new(
        device_context: &DeviceContext,
        queue: &Queue,
        command_pool_def: &CommandPoolDef,
    ) -> GfxResult<Self> {
        unimplemented!()
    }

    pub(crate) fn destroy(&self, device_context: &DeviceContext) {
        unimplemented!()
    }
}

impl CommandPool {
    pub(crate) fn reset_command_pool_platform(&self) -> GfxResult<()> {
        unimplemented!()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub(crate) struct NullDescriptorHeap;
impl NullDescriptorHeap {
    pub(crate) fn new(
        device_context: &DeviceContext,
        definition: &DescriptorHeapDef,
    ) -> GfxResult<Self> {
        unimplemented!()
    }

    pub(crate) fn destroy(&self, device_context: &DeviceContext) {
        unimplemented!()
    }
}

pub(crate) struct NullDescriptorHeapPartition;

impl NullDescriptorHeapPartition {
    pub(crate) fn new(
        device_context: &DeviceContext,
        transient: bool,
        definition: &DescriptorHeapDef,
    ) -> GfxResult<Self> {
        unimplemented!()
    }

    pub(crate) fn destroy(&self, device_context: &DeviceContext) {
        unimplemented!()
    }
}

impl DescriptorHeapPartition {
    pub(crate) fn backend_reset(&self) -> GfxResult<()> {
        unimplemented!()
    }

    pub(crate) fn backend_alloc(&self, layout: &DescriptorSetLayout) -> GfxResult<DescriptorSet> {
        unimplemented!()
    }

    pub(crate) fn backend_write(
        &self,
        layout: &DescriptorSetLayout,
        descriptor_refs: &[DescriptorRef<'_>],
    ) -> GfxResult<DescriptorSetHandle> {
        unimplemented!()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub(crate) struct NullDescriptorSetLayout;

impl NullDescriptorSetLayout {
    pub(crate) fn new(
        device_context: &DeviceContext,
        descriptors: &[Descriptor],
    ) -> GfxResult<Self> {
        unimplemented!()
    }

    pub(crate) fn destroy(&self, device_context: &DeviceContext) {
        unimplemented!()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct NullDescriptorSetWriter<'frame> {
    _phantom: PhantomData<&'frame ()>,
}

impl<'frame> NullDescriptorSetWriter<'frame> {
    pub fn new(
        descriptor_set_layout: &DescriptorSetLayout,
        bump: &'frame bumpalo::Bump,
    ) -> GfxResult<Self> {
        unimplemented!()
    }
}

impl<'frame> DescriptorSetWriter<'frame> {
    #[allow(clippy::unused_self, clippy::todo)]
    pub fn backend_set_descriptors_by_index(
        &self,
        _index: u32,
        _update_datas: &[DescriptorRef<'_>],
    ) {
        unimplemented!();
    }
    pub fn backend_set_descriptors(
        &self,
        device_context: &DeviceContext,
        descriptor_refs: &[DescriptorRef<'_>],
    ) {
        unimplemented!();
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct NullFence;

impl NullFence {
    pub(crate) fn new(device_context: &DeviceContext) -> GfxResult<Self> {
        unimplemented!()
    }

    pub(crate) fn destroy(&self, device_context: &DeviceContext) {
        unimplemented!()
    }
}

impl Fence {
    pub(crate) fn backend_wait_for_fences(
        device_context: &DeviceContext,
        fence_list: &[&Self],
    ) -> GfxResult<()> {
        unimplemented!()
    }

    pub(crate) fn get_fence_status_platform(&self) -> GfxResult<FenceStatus> {
        unimplemented!()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct NullMemoryAllocation;

impl NullMemoryAllocation {
    pub(crate) fn from_buffer(
        device_context: &DeviceContext,
        buffer: &Buffer,
        alloc_def: &MemoryAllocationDef,
    ) -> Self {
        unimplemented!()
    }

    pub(crate) fn destroy(&self, device_context: &DeviceContext) {
        unimplemented!()
    }
}

impl MemoryAllocation {
    pub(crate) fn backend_map_buffer(&self, device_context: &DeviceContext) -> BufferMappingInfo {
        unimplemented!()
    }

    pub(crate) fn backend_unmap_buffer(&self, device_context: &DeviceContext) {
        unimplemented!()
    }

    pub(crate) fn backend_mapped_ptr(&self) -> *mut u8 {
        unimplemented!()
    }

    pub(crate) fn backend_size(&self) -> usize {
        unimplemented!()
    }
}

pub(crate) struct NullMemoryPagesAllocation;

impl NullMemoryPagesAllocation {
    pub fn for_sparse_buffer(
        device_context: &DeviceContext,
        buffer: &Buffer,
        page_count: u64,
    ) -> Self {
        unimplemented!()
    }

    pub fn empty_allocation() -> Self {
        unimplemented!()
    }

    pub fn destroy(&mut self, device_context: &DeviceContext) {
        unimplemented!()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub(crate) struct NullPipeline;

impl NullPipeline {
    pub fn new_graphics_pipeline(
        device_context: &DeviceContext,
        pipeline_def: &GraphicsPipelineDef<'_>,
    ) -> GfxResult<Self> {
        unimplemented!()
    }

    pub fn new_compute_pipeline(
        device_context: &DeviceContext,
        pipeline_def: &ComputePipelineDef<'_>,
    ) -> GfxResult<Self> {
        unimplemented!()
    }

    pub fn destroy(&self, device_context: &DeviceContext) {
        unimplemented!()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct NullQueue;

impl NullQueue {
    pub fn new(device_context: &DeviceContext, queue_type: QueueType) -> GfxResult<Self> {
        unimplemented!()
    }
}

impl Queue {
    pub(crate) fn backend_family_index(&self) -> u32 {
        unimplemented!()
    }

    pub fn backend_submit(
        &self,
        command_buffers: &[&CommandBuffer],
        wait_semaphores: &[&Semaphore],
        signal_semaphores: &[&Semaphore],
        signal_fence: Option<&Fence>,
    ) -> GfxResult<()> {
        unimplemented!()
    }

    pub fn backend_present(
        &self,
        device_context: &DeviceContext,
        swapchain: &Swapchain,
        wait_semaphores: &[&Semaphore],
        image_index: u32,
    ) -> GfxResult<PresentSuccessResult> {
        unimplemented!()
    }

    pub fn backend_wait_for_queue_idle(&self) -> GfxResult<()> {
        unimplemented!()
    }

    pub fn backend_commit_sparse_bindings<'a>(
        &self,
        prev_frame_semaphore: &'a Semaphore,
        unbind_pages: &[PagedBufferAllocation],
        unbind_semaphore: &'a Semaphore,
        bind_pages: &[PagedBufferAllocation],
        bind_semaphore: &'a Semaphore,
    ) -> &'a Semaphore {
        unimplemented!()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub(crate) struct NullRootSignature;

impl NullRootSignature {
    pub(crate) fn new(
        device_context: &DeviceContext,
        definition: &RootSignatureDef,
    ) -> GfxResult<Self> {
        unimplemented!()
    }

    pub(crate) fn destroy(&self, device_context: &DeviceContext) {
        unimplemented!()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub(crate) struct NullSampler;

impl NullSampler {
    pub fn new(device_context: &DeviceContext, sampler_def: &SamplerDef) -> Self {
        unimplemented!()
    }

    pub fn destroy(&self, device_context: &DeviceContext) {
        unimplemented!()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct NullSemaphore;

impl NullSemaphore {
    pub fn new(device_context: &DeviceContext) -> Self {
        unimplemented!()
    }

    pub fn destroy(&self, device_context: &DeviceContext) {
        unimplemented!()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub(crate) struct NullShaderModule;

impl NullShaderModule {
    pub fn new(device_context: &DeviceContext, data: ShaderModuleDef<'_>) -> GfxResult<Self> {
        unimplemented!()
    }

    pub fn destroy(&self, device_context: &DeviceContext) {
        unimplemented!()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct NullSwapchain;

impl NullSwapchain {
    pub fn new(
        device_context: &DeviceContext,
        raw_window_handle: &dyn HasRawWindowHandle,
        swapchain_def: &SwapchainDef,
    ) -> GfxResult<Self> {
        unimplemented!()
    }

    pub fn destroy(&mut self) {
        unimplemented!()
    }
}

impl Swapchain {
    pub(crate) fn backend_image_count(&self) -> usize {
        unimplemented!()
    }

    pub(crate) fn backend_format(&self) -> Format {
        unimplemented!()
    }

    //TODO: Return something like PresentResult?
    pub(crate) fn backend_acquire_next_image_fence(
        &mut self,
        fence: &Fence,
    ) -> GfxResult<SwapchainImage> {
        unimplemented!()
    }

    //TODO: Return something like PresentResult?
    pub(crate) fn backend_acquire_next_image_semaphore(
        &mut self,
        semaphore: &Semaphore,
    ) -> GfxResult<SwapchainImage> {
        unimplemented!()
    }

    pub(crate) fn backend_rebuild(&mut self, swapchain_def: &SwapchainDef) -> GfxResult<()> {
        unimplemented!()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub(crate) struct NullTextureView;

impl NullTextureView {
    pub(crate) fn new(texture: &Texture, view_def: &TextureViewDef) -> Self {
        unimplemented!()
    }

    pub(crate) fn destroy(&self, device_context: &DeviceContext) {
        unimplemented!()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub(crate) struct NullRawImage;

#[derive(Debug)]
pub(crate) struct NullTexture;

impl NullTexture {
    // This path is mostly so we can wrap a provided swapchain image
    pub fn from_existing(
        device_context: &DeviceContext,
        existing_image: Option<NullRawImage>,
        texture_def: &TextureDef,
    ) -> (Self, u32) {
        unimplemented!()
    }

    pub fn destroy(&mut self, device_context: &DeviceContext) {
        unimplemented!()
    }
}

impl Texture {
    pub(crate) fn backend_map_texture(
        &self,
        plane: PlaneSlice,
    ) -> GfxResult<TextureSubResource<'_>> {
        unimplemented!()
    }

    pub(crate) fn backend_unmap_texture(&self) {
        unimplemented!()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) mod backend_impl {
    pub(crate) type BackendApi = super::NullApi;
    pub(crate) type BackendInstance = super::NullInstance;
    pub(crate) type BackendDeviceContext = super::NullDeviceContext;
    pub(crate) type BackendBuffer = super::NullBuffer;
    pub(crate) type BackendCommandBuffer = super::NullCommandBuffer;
    pub(crate) type BackendCommandPool = super::NullCommandPool;
    pub(crate) type BackendDescriptorSetHandle = ();
    pub(crate) type BackendDescriptorHeap = super::NullDescriptorHeap;
    pub(crate) type BackendDescriptorHeapPartition = super::NullDescriptorHeapPartition;
    pub(crate) type BackendDescriptorSetLayout = super::NullDescriptorSetLayout;
    pub(crate) type BackendDescriptorSetWriter<'frame> = super::NullDescriptorSetWriter<'frame>;
    pub(crate) type BackendFence = super::NullFence;
    pub(crate) type BackendMemoryAllocation = super::NullMemoryAllocation;
    pub(crate) type BackendMemoryPagesAllocation = super::NullMemoryPagesAllocation;
    pub(crate) type BackendPipeline = super::NullPipeline;
    pub(crate) type BackendQueue = super::NullQueue;
    pub(crate) type BackendRootSignature = super::NullRootSignature;
    pub(crate) type BackendSampler = super::NullSampler;
    pub(crate) type BackendSemaphore = super::NullSemaphore;
    pub(crate) type BackendShaderModule = super::NullShaderModule;
    pub(crate) type BackendSwapchain = super::NullSwapchain;
    pub(crate) type BackendTextureView = super::NullTextureView;
    pub(crate) type BackendRawImage = super::NullRawImage;
    pub(crate) type BackendTexture = super::NullTexture;
}
