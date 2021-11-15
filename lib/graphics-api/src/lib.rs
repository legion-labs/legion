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

// BEGIN - Legion Labs lints v0.6
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs lints v0.6
// crate-specific exceptions:
#![allow(clippy::missing_errors_doc)]
//#![warn(missing_docs)]

pub mod backends;
pub mod error;
pub mod reflection;
pub mod types;

pub mod prelude {
    pub use crate::types::*;
    pub use crate::{
        Buffer, BufferView, CommandBuffer, CommandPool, DescriptorSetHandle, DescriptorSetLayout,
        DeviceContext, Fence, GfxResult, Queue, Sampler, Semaphore, Shader, Swapchain, Texture,
    };
}

pub use error::*;
pub use types::*;

#[cfg(feature = "vulkan")]
pub type DefaultApi = GfxApi;

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
