//! Backend implementation of the hal

pub mod empty;

#[cfg(feature = "vulkan")]
pub mod vulkan;
