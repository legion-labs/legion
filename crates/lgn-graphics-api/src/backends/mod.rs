#[cfg(feature = "vulkan")]
pub mod vulkan;

/// Null implementation of all types
#[cfg(not(feature = "vulkan"))]
pub mod null;
