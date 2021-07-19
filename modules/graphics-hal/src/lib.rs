//! Graphics Hal
//!

// BEGIN - Legion Labs lints v0.2
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::char_lit_as_u8,
    clippy::checked_conversions,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::fallible_impl_from,
    clippy::filter_map_next,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::if_let_mutex,
    clippy::implicit_clone,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_ok_or,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::mut_mut,
    clippy::mutex_integer,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::string_add_assign,
    clippy::string_lit_as_bytes,
    clippy::string_to_string,
    clippy::todo,
    clippy::trait_duplication_in_bounds,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::useless_transmute,
    clippy::use_self,
    clippy::verbose_file_reads,
    clippy::zero_sized_map_values,
    future_incompatible,
    nonstandard_style,
    broken_intra_doc_links,
    private_intra_doc_links,
    missing_crate_level_docs,
    rust_2018_idioms
)]
// END - Legion Labs standard lints v0.2
// crate-specific exceptions:
#![allow()]
//#![warn(missing_docs)]

pub mod backends;
pub mod error;
pub mod reflection;
pub mod types;

pub mod prelude {
    pub use crate::types::*;
    pub use crate::{
        Api, Buffer, CommandBuffer, CommandPool, DefaultApi, DescriptorSetArray,
        DescriptorSetHandle, DeviceContext, Fence, GfxResult, Pipeline, Queue, RootSignature,
        Sampler, Semaphore, Shader, ShaderModule, Swapchain, Texture,
    };
}

pub use error::*;
pub use types::*;

pub use backends::null;
pub use null::NullApi;

#[cfg(feature = "vulkan")]
pub use backends::vulkan;
#[cfg(feature = "vulkan")]
pub use vulkan::VulkanApi;
#[cfg(feature = "vulkan")]
pub type DefaultApi = VulkanApi;

#[cfg(not(feature = "vulkan"))]
pub type DefaultApi = NullApi;
#[cfg(not(feature = "vulkan"))]
pub type DefaultApi = NullApi;

//
// Constants
//

/// The maximum descriptor set layout index allowed. Vulkan only guarantees up to 4 are available
pub const MAX_DESCRIPTOR_SET_LAYOUTS: usize = 4;
/// The maximum number of simultaneously attached render targets
// In sync with BlendStateTargets
pub const MAX_RENDER_TARGET_ATTACHMENTS: usize = 8;
// Vulkan guarantees up to 16
pub const MAX_VERTEX_INPUT_BINDINGS: usize = 16;

//
// Root of the API
//
pub trait Api: Sized {
    fn device_context(&self) -> &Self::DeviceContext;
    fn destroy(&mut self) -> GfxResult<()>;

    type DeviceContext: DeviceContext<Self>;
    type Buffer: Buffer<Self>;
    type Texture: Texture<Self>;
    type Sampler: Sampler<Self>;
    type ShaderModule: ShaderModule<Self>;
    type Shader: Shader<Self>;
    type RootSignature: RootSignature<Self>;
    type Pipeline: Pipeline<Self>;
    type DescriptorSetHandle: DescriptorSetHandle<Self>;
    type DescriptorSetArray: DescriptorSetArray<Self>;
    type Queue: Queue<Self>;
    type CommandPool: CommandPool<Self>;
    type CommandBuffer: CommandBuffer<Self>;
    type Fence: Fence<Self>;
    type Semaphore: Semaphore<Self>;
    type Swapchain: Swapchain<Self>;
}

pub trait DeviceContext<A: Api>: Clone {
    fn device_info(&self) -> &DeviceInfo;
    fn create_queue(&self, queue_type: QueueType) -> GfxResult<A::Queue>;
    fn create_fence(&self) -> GfxResult<A::Fence>;
    fn create_semaphore(&self) -> GfxResult<A::Semaphore>;
    fn create_swapchain(
        &self,
        raw_window_handle: &dyn raw_window_handle::HasRawWindowHandle,
        swapchain_def: &SwapchainDef,
    ) -> GfxResult<A::Swapchain>;
    fn create_sampler(&self, sampler_def: &SamplerDef) -> GfxResult<A::Sampler>;
    fn create_texture(&self, texture_def: &TextureDef) -> GfxResult<A::Texture>;
    fn create_buffer(&self, buffer_def: &BufferDef) -> GfxResult<A::Buffer>;
    fn create_shader(&self, stages: Vec<ShaderStageDef<A>>) -> GfxResult<A::Shader>;
    fn create_root_signature(
        &self,
        root_signature_def: &RootSignatureDef<'_, A>,
    ) -> GfxResult<A::RootSignature>;
    fn create_descriptor_set_array(
        &self,
        descriptor_set_array_def: &DescriptorSetArrayDef<'_, A>,
    ) -> GfxResult<A::DescriptorSetArray>;
    fn create_graphics_pipeline(
        &self,
        graphics_pipeline_def: &GraphicsPipelineDef<'_, A>,
    ) -> GfxResult<A::Pipeline>;
    fn create_compute_pipeline(
        &self,
        compute_pipeline_def: &ComputePipelineDef<'_, A>,
    ) -> GfxResult<A::Pipeline>;
    fn create_shader_module(&self, data: ShaderModuleDef<'_>) -> GfxResult<A::ShaderModule>;

    fn wait_for_fences(&self, fences: &[&A::Fence]) -> GfxResult<()>;

    fn find_supported_format(
        &self,
        candidates: &[Format],
        resource_type: ResourceType,
    ) -> Option<Format>;
    fn find_supported_sample_count(&self, candidates: &[SampleCount]) -> Option<SampleCount>;
}

//
// Resources (Buffers, Textures, Samplers)
//
pub trait Buffer<A: Api>: std::fmt::Debug {
    fn buffer_def(&self) -> &BufferDef;
    fn map_buffer(&self) -> GfxResult<*mut u8>;
    fn unmap_buffer(&self) -> GfxResult<()>;
    fn mapped_memory(&self) -> Option<*mut u8>;
    fn copy_to_host_visible_buffer<T: Copy>(&self, data: &[T]) -> GfxResult<()>;
    fn copy_to_host_visible_buffer_with_offset<T: Copy>(
        &self,
        data: &[T],
        buffer_byte_offset: u64,
    ) -> GfxResult<()>;
}
pub trait Texture<A: Api>: Clone + std::fmt::Debug {
    fn texture_def(&self) -> &TextureDef;
}
pub trait Sampler<A: Api>: Clone + std::fmt::Debug {}

//
// Shaders/Pipelines
//
pub trait ShaderModule<A: Api>: Clone + std::fmt::Debug {}

pub trait Shader<A: Api>: Clone + std::fmt::Debug {
    fn pipeline_reflection(&self) -> &PipelineReflection;
}

pub trait RootSignature<A: Api>: Clone + std::fmt::Debug {
    fn pipeline_type(&self) -> PipelineType;
}

pub trait Pipeline<A: Api>: std::fmt::Debug {
    fn pipeline_type(&self) -> PipelineType;
    fn root_signature(&self) -> &A::RootSignature;
}

//
// Descriptor Sets
//
pub trait DescriptorSetHandle<A: Api>: std::fmt::Debug {}

pub trait DescriptorSetArray<A: Api>: std::fmt::Debug {
    fn handle(&self, array_index: u32) -> Option<A::DescriptorSetHandle>;
    fn root_signature(&self) -> &A::RootSignature;
    fn update_descriptor_set(&mut self, params: &[DescriptorUpdate<'_, A>]) -> GfxResult<()>;
    fn queue_descriptor_set_update(&mut self, update: &DescriptorUpdate<'_, A>) -> GfxResult<()>;
    fn flush_descriptor_set_updates(&mut self) -> GfxResult<()>;
}

//
// Queues, Command Buffers
//
pub trait Queue<A: Api>: Clone + std::fmt::Debug {
    fn device_context(&self) -> &A::DeviceContext;
    fn queue_id(&self) -> u32;
    fn queue_type(&self) -> QueueType;
    fn create_command_pool(&self, command_pool_def: &CommandPoolDef) -> GfxResult<A::CommandPool>;
    fn submit(
        &self,
        command_buffers: &[&A::CommandBuffer],
        wait_semaphores: &[&A::Semaphore],
        signal_semaphores: &[&A::Semaphore],
        signal_fence: Option<&A::Fence>,
    ) -> GfxResult<()>;
    fn present(
        &self,
        swapchain: &A::Swapchain,
        wait_semaphores: &[&A::Semaphore],
        image_index: u32,
    ) -> GfxResult<PresentSuccessResult>;
    fn wait_for_queue_idle(&self) -> GfxResult<()>;
}

pub trait CommandPool<A: Api> {
    fn device_context(&self) -> &A::DeviceContext;
    fn create_command_buffer(
        &self,
        command_buffer_def: &CommandBufferDef,
    ) -> GfxResult<A::CommandBuffer>;
    fn reset_command_pool(&self) -> GfxResult<()>;
}

pub trait CommandBuffer<A: Api>: std::fmt::Debug {
    fn begin(&self) -> GfxResult<()>;
    fn end(&self) -> GfxResult<()>;
    fn return_to_pool(&self) -> GfxResult<()>;

    fn cmd_begin_render_pass(
        &self,
        color_targets: &[ColorRenderTargetBinding<'_, A>],
        depth_target: Option<DepthStencilRenderTargetBinding<'_, A>>,
    ) -> GfxResult<()>;
    fn cmd_end_render_pass(&self) -> GfxResult<()>;

    fn cmd_set_viewport(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        depth_min: f32,
        depth_max: f32,
    ) -> GfxResult<()>;
    fn cmd_set_scissor(&self, x: u32, y: u32, width: u32, height: u32) -> GfxResult<()>;
    fn cmd_set_stencil_reference_value(&self, value: u32) -> GfxResult<()>;
    fn cmd_bind_pipeline(&self, pipeline: &A::Pipeline) -> GfxResult<()>;
    fn cmd_bind_vertex_buffers(
        &self,
        first_binding: u32,
        bindings: &[VertexBufferBinding<'_, A>],
    ) -> GfxResult<()>;
    fn cmd_bind_index_buffer(&self, binding: &IndexBufferBinding<'_, A>) -> GfxResult<()>;
    fn cmd_bind_descriptor_set(
        &self,
        descriptor_set_array: &A::DescriptorSetArray,
        index: u32,
    ) -> GfxResult<()>;
    fn cmd_bind_descriptor_set_handle(
        &self,
        root_signature: &A::RootSignature,
        set_index: u32,
        descriptor_set_handle: &A::DescriptorSetHandle,
    ) -> GfxResult<()>;

    fn cmd_draw(&self, vertex_count: u32, first_vertex: u32) -> GfxResult<()>;
    fn cmd_draw_instanced(
        &self,
        vertex_count: u32,
        first_vertex: u32,
        instance_count: u32,
        first_instance: u32,
    ) -> GfxResult<()>;
    fn cmd_draw_indexed(
        &self,
        index_count: u32,
        first_index: u32,
        vertex_offset: i32,
    ) -> GfxResult<()>;
    fn cmd_draw_indexed_instanced(
        &self,
        index_count: u32,
        first_index: u32,
        instance_count: u32,
        first_instance: u32,
        vertex_offset: i32,
    ) -> GfxResult<()>;

    fn cmd_dispatch(
        &self,
        group_count_x: u32,
        group_count_y: u32,
        group_count_z: u32,
    ) -> GfxResult<()>;

    fn cmd_resource_barrier(
        &self,
        buffer_barriers: &[BufferBarrier<'_, A>],
        texture_barriers: &[TextureBarrier<'_, A>],
    ) -> GfxResult<()>;
    fn cmd_copy_buffer_to_buffer(
        &self,
        src_buffer: &A::Buffer,
        dst_buffer: &A::Buffer,
        src_offset: u64,
        dst_offset: u64,
        size: u64,
    ) -> GfxResult<()>;
    fn cmd_copy_buffer_to_texture(
        &self,
        src_buffer: &A::Buffer,
        dst_texture: &A::Texture,
        params: &CmdCopyBufferToTextureParams,
    ) -> GfxResult<()>;
    fn cmd_blit_image(
        &self,
        src_texture: &A::Texture,
        dst_texture: &A::Texture,
        params: &CmdBlitParams,
    ) -> GfxResult<()>;
}

//
// Fences and Semaphores
//
pub trait Fence<A: Api> {
    fn wait(&self) -> GfxResult<()>;
    fn wait_for_fences(device_context: &A::DeviceContext, fences: &[&A::Fence]) -> GfxResult<()>;
    fn get_fence_status(&self) -> GfxResult<FenceStatus>;
}

pub trait Semaphore<A: Api> {}

//
// Swapchain
//
pub trait Swapchain<A: Api> {
    fn swapchain_def(&self) -> &SwapchainDef;
    fn image_count(&self) -> usize;
    fn format(&self) -> Format;
    fn acquire_next_image_fence(&mut self, fence: &A::Fence) -> GfxResult<SwapchainImage<A>>;
    fn acquire_next_image_semaphore(
        &mut self,
        semaphore: &A::Semaphore,
    ) -> GfxResult<SwapchainImage<A>>;
    fn rebuild(&mut self, swapchain_def: &SwapchainDef) -> GfxResult<()>;
}
