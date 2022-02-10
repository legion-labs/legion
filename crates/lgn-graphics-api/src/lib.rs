//! Graphics Api

// crate-specific lint exceptions:
#![allow(clippy::missing_errors_doc)]
//#![warn(missing_docs)]
#![cfg_attr(not(any(feature = "vulkan")), allow(dead_code))]

pub mod backends;
pub mod error;
pub mod reflection;
pub mod types;

mod api;
pub use api::*;

mod buffer;
pub use buffer::*;

mod buffer_view;
pub use buffer_view::*;

mod command_buffer;
pub use command_buffer::*;

mod command_pool;
pub use command_pool::*;

mod descriptor_heap;
pub use descriptor_heap::*;

mod descriptor_set_layout;
pub use descriptor_set_layout::*;

mod descriptor_set_writer;
pub use descriptor_set_writer::*;

mod device_context;
pub use device_context::*;

mod fence;
pub use fence::*;

mod memory_allocation;
pub use memory_allocation::*;

mod pipeline;
pub use pipeline::*;

mod queue;
pub use queue::*;

mod root_signature;
pub use root_signature::*;

mod sampler;
pub use sampler::*;

mod shader;
pub use shader::*;

mod shader_module;
pub use shader_module::*;

mod semaphore;
pub use semaphore::*;

mod swapchain;
pub use swapchain::*;

mod texture;
pub use texture::*;

mod texture_view;
pub use texture_view::*;

pub mod prelude {
    pub use crate::types::*;
    pub use crate::*;
    pub use crate::{
        Buffer, BufferView, CommandBuffer, CommandPool, DescriptorSetHandle, DescriptorSetLayout,
        DeviceContext, Fence, GfxResult, Queue, Sampler, Semaphore, Shader, Swapchain, Texture,
    };
}

pub use error::*;
pub use types::*;

//
// Constants
//

/// The maximum descriptor set layout index allowed. Vulkan only guarantees up
/// to 4 are available
pub const MAX_DESCRIPTOR_SET_LAYOUTS: usize = 4;
pub const MAX_DESCRIPTOR_BINDINGS: usize = 64;
/// The maximum number of simultaneously attached render targets
// In sync with BlendStateTargets
pub const MAX_RENDER_TARGET_ATTACHMENTS: usize = 8;
// Vulkan guarantees up to 16
pub const MAX_VERTEX_INPUT_BINDINGS: usize = 16;
