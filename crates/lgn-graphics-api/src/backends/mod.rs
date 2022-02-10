#[cfg(feature = "vulkan")]
pub mod vulkan;
#[cfg(feature = "vulkan")]
pub(crate) use vulkan::backend_impl::*;

/// Null implementation of all types
#[cfg(not(feature = "vulkan"))]
pub mod null;
#[cfg(not(feature = "vulkan"))]
pub(crate) use null::backend_impl::*;
