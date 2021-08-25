use std::sync::Arc;

pub type GfxResult<T> = Result<T, GfxError>;

/// Generic error that contains all the different kinds of errors that may occur when using the API
#[derive(Debug, Clone)]
pub enum GfxError {
    StringError(String),
    IoError(Arc<std::io::Error>),
    #[cfg(feature = "vulkan")]
    VkError(ash::vk::Result),
    #[cfg(feature = "vulkan")]
    VkLoadingError(Arc<ash::LoadingError>),
    #[cfg(feature = "vulkan")]
    VkCreateInstanceError(Arc<crate::vulkan::VkCreateInstanceError>),
    #[cfg(feature = "vulkan")]
    VkMemError(Arc<vk_mem::Error>),
}

impl From<&str> for GfxError {
    fn from(str: &str) -> Self {
        Self::StringError(str.to_string())
    }
}

impl From<String> for GfxError {
    fn from(string: String) -> Self {
        Self::StringError(string)
    }
}

impl From<std::io::Error> for GfxError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(Arc::new(error))
    }
}

#[cfg(feature = "vulkan")]
impl From<ash::vk::Result> for GfxError {
    fn from(result: ash::vk::Result) -> Self {
        Self::VkError(result)
    }
}

#[cfg(feature = "vulkan")]
impl From<ash::LoadingError> for GfxError {
    fn from(result: ash::LoadingError) -> Self {
        Self::VkLoadingError(Arc::new(result))
    }
}

#[cfg(feature = "vulkan")]
impl From<crate::vulkan::VkCreateInstanceError> for GfxError {
    fn from(result: crate::vulkan::VkCreateInstanceError) -> Self {
        Self::VkCreateInstanceError(Arc::new(result))
    }
}

#[cfg(feature = "vulkan")]
impl From<vk_mem::Error> for GfxError {
    fn from(error: vk_mem::Error) -> Self {
        Self::VkMemError(Arc::new(error))
    }
}
