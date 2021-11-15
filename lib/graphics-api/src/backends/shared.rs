#[cfg(any(feature = "vulkan"))]
pub(crate) static NEXT_TEXTURE_ID: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(1);
