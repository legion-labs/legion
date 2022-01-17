use std::{error::Error, sync::Arc};

pub type GfxResult<T> = Result<T, GfxError>;

/// Generic error that contains all the different kinds of errors that may occur
/// when using the API
#[derive(Debug, Clone)]
pub enum GfxError {
    String(String),
    Io(Arc<std::io::Error>),
    #[cfg(feature = "vulkan")]
    Vk(ash::vk::Result),
    #[cfg(feature = "vulkan")]
    VkMem(Arc<vk_mem::Error>),
}

impl std::fmt::Display for GfxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GfxError::String(msg) => write!(f, "{}", msg),
            GfxError::Io(e) => e.fmt(f),
            #[cfg(feature = "vulkan")]
            GfxError::Vk(e) => e.fmt(f),
            #[cfg(feature = "vulkan")]
            GfxError::VkMem(e) => e.fmt(f),
        }
    }
}

impl Error for GfxError {}

impl From<&str> for GfxError {
    fn from(str: &str) -> Self {
        Self::String(str.to_string())
    }
}

impl From<String> for GfxError {
    fn from(string: String) -> Self {
        Self::String(string)
    }
}

impl From<std::io::Error> for GfxError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(Arc::new(error))
    }
}

#[cfg(feature = "vulkan")]
impl From<ash::vk::Result> for GfxError {
    fn from(result: ash::vk::Result) -> Self {
        Self::Vk(result)
    }
}

#[cfg(feature = "vulkan")]
impl From<vk_mem::Error> for GfxError {
    fn from(error: vk_mem::Error) -> Self {
        Self::VkMem(Arc::new(error))
    }
}
