//! Graphics Api is an abstraction layer focussed on providing a bleeding edge real-time rendering
//! engine with the necessary abstraction when interacting with the different Api. It is focussed in
//! the sense that it does not cover 100% of what the lower level apis offer.
//! Current and future design choices are driven by the following:
//!  * High performance realtime focussed, at the expense of ease of use
//!  * Focusses of the needs of Legion, with special attention to server side rendering, gpu based pipeline
//!
//! Additional Resources:
//! * Tracking Issue: [legion/crate/#64](https://github.com/legion-labs/legion/issues/64)
//! * Design Doc: [legion/book/rendering](/book/rendering/graphics-api.html)
//!
//! The implementation of Graphics Api is based of `Rafx`, but instead of using concrete enum types
//! to represents the different backends, the Api uses an Api trait that is expected to be used
//! for static dispatch, so everything interacting with graphics will carry an additional generic
//! parameter.
//!
//! Why fork `Rafx` instead of using it directly? The initial commit actually of how the library
//! interfaces with the rest of our ecosystem, and as we will be investing more and more, we hardly
//! see ourself depend on an out of repo implementation of low level graphics. We will try to contribute
//! ideas, fixes when applicable back to Rafx.
//!
//! The Api does not track resource lifetimes or states (such as vulkan image layouts) or try to
//! enforce safe usage at compile time or runtime.
//!
//! **Every API call is potentially unsafe.** However, the unsafe keyword is only placed on APIs
//! that are particularly likely to cause undefined behavior if used incorrectly.
//!
//! `Rafx` general shape of the Api is inspired by [The Forge](https://github.com/ConfettiFX/The-Forge).
//! It was chosen for its modern design, multiple working backends, open development model, and track
//! record of shipped games. However, there are some changes in API design, feature set,
//! and implementation details.
//!
//! # Main Api Objects
//!
//! * [`GfxApi`] - Primary entry point to using the API. Use the new_* functions to initialize the desired backend.
//! * [`Buffer`] - Memory that can be accessed by the rendering API. It may reside in CPU or GPU memory.
//! * [`CommandBuffer`] - A list of commands recorded by the CPU and submitted to the GPU.
//! * [`CommandPool`] - A pool of command buffers. A command pool is necessary to create a command buffer.
//! * [`DescriptorSetArray`] - An array of descriptor sets. These are expected to be pooled and reused.
//! * [`DeviceContext`] - A cloneable, thread-safe handle used to create graphics resources.
//! * [`Fence`] - A GPU -> CPU synchronization mechanism.
//! * [`Pipeline`] - Represents a complete GPU configuration for executing work.
//! * [`Queue`] - A queue allows work to be submitted to the GPU
//! * [`RootSignature`] - Represents the full "layout" or "interface" of a shader (or set of shaders.)
//! * [`Sampler`] - Configures how images will be sampled by the GPU
//! * [`Semaphore`] - A GPU -> GPU synchronization mechanism.
//! * [`Shader`] - Represents one or more shader stages, producing an entire "program" to execute on the GPU
//! * [`ShaderModule`] - Rrepresents loaded shader code that can be used to create a pipeline.
//! * [`Swapchain`] - A set of images that act as a "backbuffer" of a window.
//! * [`Texture`] - An image that can be used by the GPU.
//!
//! # Usage Summary
//!
//! In order to interact with a graphics API, construct a `Api`. A different new_* function
//! exists for each backend.
//!
//! ```ignore
//! let api = DefaultApi::new(...);
//! ```
//!
//! After initialization, most interaction will be via `DeviceContext` Call
//! `Api::device_context()` on the the api object to obtain a cloneable handle that can be
//! used from multiple threads.
//!
//! ```ignore
//! let device_context = api.device_context();
//! ```
//!
//! Most objects are created via `DeviceContext`. For example:
//!
//! ```ignore
//! // (See examples for more detail here!)
//! let texture = device_context.create_texture(...)?;
//! let buffer = device_context.create_buffer(...)?;
//! let shader_module = device_context.create_shader_module(...)?;
//! ```
//!
//! In order to submit work to the GPU, a `CommandBuffer` must be submitted to a `Queue`.
//! Most commonly, this needs to be a "Graphics" queue.
//!
//! Obtaining a `Queue` is straightforward. Here we will get a "Graphics" queue. This queue type
//! supports ALL operations (including compute) and is usually the correct one to use if you aren't
//! sure.
//!
//! ```ignore
//! let queue = device_context.create_queue(QueueType::Graphics)?;
//! ```
//!
//! A command buffer cannot be created directly. It must be allocated out of a pool.
//!
//! The command pool and all command buffers allocated from it share memory. The standard rust rules
//! about mutability apply but are not enforced at compile time or runtime.
//!  * Do not modify two command buffers from the same pool concurrently
//!  * Do not allocate from a command pool while modifying one of its command buffers
//!  * Once a command buffer is submitted to the GPU, do not modify its pool, or any command buffers
//!    created from it, until the GPU completes its work.
//!
//! In general, do not modify textures, buffers, command buffers, or other GPU resources while a
//! command buffer referencing them is submitted. Additionally, these resources must persist for
//! the entire duration of the submitted workload.
//!
//! ```ignore
//! let command_pool = queue.create_command_pool(&CommandPoolDef {
//!     transient: true
//! })?;
//!
//! let command_buffer = command_pool.create_command_buffer(&CommandBufferDef {
//!     is_secondary: false,
//! })?;
//! ```
//!
//! Once a command buffer is obtained, write to it by calling "cmd" functions on it, For example,
//! drawing primitives looks like this. Call begin() before writing to it, and end() after finished
//! writing to it.
//!
//! ```ignore
//! command_buffer.begin()?;
//! // other setup...
//! command_buffer.cmd_draw(3, 0)?;
//! command_buffer.end()?;
//! ```
//!
//! For the most part, no actual work is performed when calling these functions. We are just
//! "scheduling" work to happen later when we give the command buffer to the GPU.
//!
//! After writing the command buffer, it must be submitted to the queue. The "scheduled" work
//! described in the command buffer will happen asynchronously from the rest of the program.
//!
//! ```ignore
//! queue.submit(
//!     &[&command_buffer],
//!     &[], // No semaphores or fences in this example
//!     &[],
//!     None
//! )?;
//! queue.wait_for_queue_idle()?;
//! ```
//!
//! The command buffer, the command pool it was allocated from, all other command buffers allocated
//! from that pool, and any other resources referenced by this command buffer cannot be dropped
//! until the queued work is complete, and generally speaking must remain immutable.
//!
//! More fine-grained synchronization is available via Fence and Semaphore but that will
//! not be covered here.
//!
//! # Resource Barriers
//!
//! CPUs generally provide a single "coherent" view of memory, but this is not the case for GPUs.
//! Resources can also be stored in many forms depending on how they are used. (The details of this
//! are device-specific and outside the scope of these docs). Resources must be placed into an
//! appropriate state to use them.
//!
//! Additionally modifying a resource (or transitioning its state) can result in memory hazards. A
//! memory hazard is when reading/writing to memory occurs in an undefined order, resulting in
//! undefined behavior.
//!
//! `Barriers` are used to transition resources into the correct state and to avoid these hazards.
//! Here is an example where we take an image from the swapchain and prepare it for use.
//! (We will also need a barrier after we modify it to transition it back to PRESENT!)
//!
//! ```ignore
//! command_buffer.cmd_resource_barrier(
//!     &[], // no buffers to transition
//!     &[
//!         // Transition `texture` from PRESENT state to RENDER_TARGET state
//!         TextureBarrier::state_transition(
//!             &texture,
//!             ResourceState::PRESENT,
//!             ResourceState::RENDER_TARGET,
//!         )
//!     ],
//! )?;
//! ```
//!
//! # "Definition" structs
//!
//! Many functions take a "def" parameter. For example, `DeviceContext::create_texture()` takes
//! a single `TextureDef` parameter. Here is an example call:
//!
//! ```ignore
//!     let texture = device_context.create_texture(&TextureDef {
//!         extents: Extents3D {
//!             width: 512,
//!             height: 512,
//!             depth: 1,
//!         },
//!         array_length: 1,
//!         mip_count: 1,
//!         sample_count: SampleCount::SampleCount1,
//!         format: Format::R8G8B8A8_UNORM,
//!         resource_type: ResourceType::TEXTURE,
//!         dimensions: TextureDimensions::Dim2D,
//!     })?;
//! ```
//!
//! There are advantages to this approach:
//! * The code is easier to read - parameters are clearly labeled
//! * Default values can be used
//! * When new "parameters" are added, if Default is used, the code will still compile. This avoids
//!   boilerplate to implement the builder pattern
//!
//! ```ignore
//!     let texture = device_context.create_texture(&TextureDef {
//!         extents: Extents3D {
//!             width: 512,
//!             height: 512,
//!             depth: 1,
//!         },
//!         format: Format::R8G8B8A8_UNORM,
//!         ..Default::default()
//!     })?;
//! ```
//!
//!

// BEGIN - Legion Labs lints v0.3
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
    rust_2018_idioms,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links,
    rustdoc::broken_intra_doc_links
)]
// END - Legion Labs standard lints v0.3
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
        Buffer, CommandBuffer, CommandPool, DefaultApi, DescriptorSetArray, DescriptorSetHandle,
        DescriptorSetLayout, DeviceContext, Fence, GfxApi, GfxResult, Pipeline, Queue,
        RootSignature, Sampler, Semaphore, Shader, ShaderModule, Swapchain, Texture,
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
pub trait GfxApi: Sized {
    fn device_context(&self) -> &Self::DeviceContext;
    fn destroy(&mut self) -> GfxResult<()>;

    type DeviceContext: DeviceContext<Self>;
    type Buffer: Buffer<Self>;
    type Texture: Texture<Self>;
    type Sampler: Sampler<Self>;
    type ShaderModule: ShaderModule<Self>;
    type Shader: Shader<Self>;
    type DescriptorSetLayout: DescriptorSetLayout<Self>;
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

pub trait DeviceContext<A: GfxApi>: Clone {
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
    fn create_descriptorset_layout(
        &self,
        def: &DescriptorSetLayoutDef,
    ) -> GfxResult<A::DescriptorSetLayout>;
    fn create_root_signature(
        &self,
        root_signature_def: &RootSignatureDef<A>,
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
pub trait Buffer<A: GfxApi>: std::fmt::Debug {
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

pub trait Texture<A: GfxApi>: Clone + std::fmt::Debug {
    fn texture_def(&self) -> &TextureDef;
    fn map_texture(&self) -> GfxResult<TextureSubResource<'_>>;
    fn unmap_texture(&self) -> GfxResult<()>;
}

pub trait Sampler<A: GfxApi>: Clone + std::fmt::Debug {}

//
// Shaders/Pipelines
//
pub trait ShaderModule<A: GfxApi>: Clone + std::fmt::Debug {}

pub trait Shader<A: GfxApi>: Clone + std::fmt::Debug {
    fn pipeline_reflection(&self) -> &PipelineReflection;
}

pub trait DescriptorSetLayout<A: GfxApi>: Clone + std::fmt::Debug {
    fn pipeline_type(&self) -> PipelineType;
}

pub trait RootSignature<A: GfxApi>: Clone + std::fmt::Debug {
    fn pipeline_type(&self) -> PipelineType;
}

pub trait Pipeline<A: GfxApi>: std::fmt::Debug {
    fn pipeline_type(&self) -> PipelineType;
    fn root_signature(&self) -> &A::RootSignature;
}

//
// Descriptor Sets
//
pub trait DescriptorSetHandle<A: GfxApi>: std::fmt::Debug {}

pub trait DescriptorSetArray<A: GfxApi>: std::fmt::Debug {
    fn handle(&self, array_index: u32) -> Option<A::DescriptorSetHandle>;
    fn update_descriptor_set(&mut self, params: &[DescriptorUpdate<'_, A>]) -> GfxResult<()>;
    fn queue_descriptor_set_update(&mut self, update: &DescriptorUpdate<'_, A>) -> GfxResult<()>;
    fn flush_descriptor_set_updates(&mut self) -> GfxResult<()>;
}

//
// Queues, Command Buffers
//
pub trait Queue<A: GfxApi>: Clone + std::fmt::Debug {
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

pub trait CommandPool<A: GfxApi> {
    fn device_context(&self) -> &A::DeviceContext;
    fn create_command_buffer(
        &self,
        command_buffer_def: &CommandBufferDef,
    ) -> GfxResult<A::CommandBuffer>;
    fn reset_command_pool(&self) -> GfxResult<()>;
}

pub trait CommandBuffer<A: GfxApi>: std::fmt::Debug {
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
        root_signature: &A::RootSignature,
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
    fn cmd_blit_texture(
        &self,
        src_texture: &A::Texture,
        dst_texture: &A::Texture,
        params: &CmdBlitParams,
    ) -> GfxResult<()>;
    fn cmd_copy_image(
        &self,
        src_texture: &A::Texture,
        dst_texture: &A::Texture,
        params: &CmdCopyTextureParams,
    ) -> GfxResult<()>;
}

//
// Fences and Semaphores
//
pub trait Fence<A: GfxApi> {
    fn wait(&self) -> GfxResult<()>;
    fn wait_for_fences(device_context: &A::DeviceContext, fences: &[&A::Fence]) -> GfxResult<()>;
    fn get_fence_status(&self) -> GfxResult<FenceStatus>;
}

pub trait Semaphore<A: GfxApi> {}

//
// Swapchain
//
pub trait Swapchain<A: GfxApi> {
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
