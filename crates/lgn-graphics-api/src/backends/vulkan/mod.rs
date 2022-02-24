#![allow(unsafe_code)]

mod api;
pub(crate) use api::*;

mod buffer;
pub(crate) use buffer::*;

mod command_buffer;
pub(crate) use command_buffer::*;

mod command_pool;
pub(crate) use command_pool::*;

mod descriptor_heap;
pub(crate) use descriptor_heap::*;

mod descriptor_set_layout;
pub(crate) use descriptor_set_layout::*;

mod descriptor_set_writer;
pub use descriptor_set_writer::*;

mod device_context;
pub(crate) use device_context::*;

mod fence;
pub(crate) use fence::*;

mod internal;
pub(crate) use internal::*;

mod memory_allocation;
pub(crate) use memory_allocation::*;

mod pipeline;
pub(crate) use pipeline::*;

mod queue;
pub(crate) use queue::*;

mod root_signature;
pub(crate) use root_signature::*;

mod sampler;
pub(crate) use sampler::*;

mod semaphore;
pub(crate) use semaphore::*;

mod shader_module;
pub(crate) use shader_module::*;

mod swapchain;
pub(crate) use swapchain::*;

mod texture_view;
pub(crate) use texture_view::*;

mod texture;
pub(crate) use texture::*;

mod video;

pub(crate) mod backend_impl {
    pub(crate) type BackendApi = super::VulkanApi;
    pub(crate) type BackendInstance = super::VkInstance;
    pub(crate) type BackendDeviceContext = super::VulkanDeviceContext;
    pub(crate) type BackendBuffer = super::VulkanBuffer;
    pub(crate) type BackendCommandBuffer = super::VulkanCommandBuffer;
    pub(crate) type BackendCommandPool = super::VulkanCommandPool;
    pub(crate) type BackendDescriptorSetHandle = ash::vk::DescriptorSet;
    pub(crate) type BackendDescriptorHeap = super::VulkanDescriptorHeap;
    pub(crate) type BackendDescriptorHeapPartition = super::VulkanDescriptorHeapPartition;
    pub(crate) type BackendDescriptorSetLayout = super::VulkanDescriptorSetLayout;
    pub(crate) type BackendFence = super::VulkanFence;
    pub(crate) type BackendMemoryAllocation = super::VulkanMemoryAllocation;
    pub(crate) type BackendMemoryPagesAllocation = super::VulkanMemoryPagesAllocation;
    pub(crate) type BackendPipeline = super::VulkanPipeline;
    pub(crate) type BackendQueue = super::VulkanQueue;
    pub(crate) type BackendRootSignature = super::VulkanRootSignature;
    pub(crate) type BackendSampler = super::VulkanSampler;
    pub(crate) type BackendSemaphore = super::VulkanSemaphore;
    pub(crate) type BackendShaderModule = super::VulkanShaderModule;
    pub(crate) type BackendSwapchain = super::VulkanSwapchain;
    pub(crate) type BackendTextureView = super::VulkanTextureView;
    pub(crate) type BackendTexture = super::VulkanTexture;
    pub(crate) type BackendRawImage = super::VulkanRawImage;
}
