use raw_window_handle::HasRawWindowHandle;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use super::deferred_drop::DeferredDropper;
use crate::{
    ApiDef, Buffer, BufferDef, ComputePipelineDef, DescriptorHeap, DescriptorHeapDef,
    DescriptorSetLayout, DescriptorSetLayoutDef, ExtensionMode, Fence, GfxResult,
    GraphicsPipelineDef, Instance, Pipeline, PipelineReflection, Queue, QueueType, RootSignature,
    RootSignatureDef, Sampler, SamplerDef, Semaphore, Shader, ShaderModule, ShaderModuleDef,
    ShaderStageDef, Swapchain, SwapchainDef, Texture, TextureDef,
};

#[cfg(any(feature = "vulkan"))]
use crate::DeviceInfo;

#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VulkanDeviceContext;

/// Used to specify which type of physical device is preferred. It's recommended to read the Vulkan
/// spec to understand precisely what these types mean
///
/// Values here match `VkPhysicalDeviceType`, `DiscreteGpu` is the recommended default
#[derive(Copy, Clone, Debug)]
pub enum PhysicalDeviceType {
    /// Corresponds to `VK_PHYSICAL_DEVICE_TYPE_OTHER`
    Other = 0,

    /// Corresponds to `VK_PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU`
    IntegratedGpu = 1,

    /// Corresponds to `VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU`
    DiscreteGpu = 2,

    /// Corresponds to `VK_PHYSICAL_DEVICE_TYPE_VIRTUAL_GPU`
    VirtualGpu = 3,

    /// Corresponds to `VK_PHYSICAL_DEVICE_TYPE_CPU`
    Cpu = 4,
}

pub(crate) struct DeviceContextInner {
    #[cfg(any(feature = "vulkan"))]
    device_info: DeviceInfo,
    pub(crate) deferred_dropper: DeferredDropper,
    destroyed: AtomicBool,

    #[cfg(debug_assertions)]
    #[cfg(feature = "track-device-contexts")]
    next_create_index: AtomicU64,

    #[cfg(debug_assertions)]
    #[cfg(feature = "track-device-contexts")]
    all_contexts: Mutex<fnv::FnvHashMap<u64, backtrace::Backtrace>>,

    #[cfg(feature = "vulkan")]
    pub(crate) platform_device_context: VulkanDeviceContext,
}

impl std::fmt::Debug for DeviceContextInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VulkanDeviceContext")
            .field(
                "handle",
                #[cfg(any(feature = "vulkan"))]
                &self.platform_device_context.device().handle(),
                #[cfg(not(any(feature = "vulkan")))]
                &0,
            )
            .finish()
    }
}

impl Drop for DeviceContextInner {
    fn drop(&mut self) {
        if !self.destroyed.swap(true, Ordering::AcqRel) {
            log::trace!("destroying device");
            self.deferred_dropper.destroy();

            #[cfg(any(feature = "vulkan"))]
            self.platform_device_context.destroy();

            //self.surface_loader.destroy_surface(self.surface, None);
            log::trace!("destroyed device");
        }
    }
}

impl DeviceContextInner {
    pub fn new(
        #[cfg(not(any(feature = "vulkan")))] instance: &Instance,
        #[cfg(any(feature = "vulkan"))] instance: &Instance<'_>,
        windowing_mode: ExtensionMode,
        video_mode: ExtensionMode,
    ) -> GfxResult<Self> {
        #[cfg(debug_assertions)]
        #[cfg(feature = "track-device-contexts")]
        let all_contexts = {
            let create_backtrace = backtrace::Backtrace::new_unresolved();
            let mut all_contexts = fnv::FnvHashMap::<u64, backtrace::Backtrace>::default();
            all_contexts.insert(0, create_backtrace);
            all_contexts
        };

        #[cfg(feature = "vulkan")]
        let (platform_device_context, device_info) =
            VulkanDeviceContext::new(instance.platform_instance, windowing_mode, video_mode)
                .map_err(|e| {
                    log::error!("Error creating device context {:?}", e);
                    ash::vk::Result::ERROR_UNKNOWN
                })?;

        Ok(Self {
            #[cfg(any(feature = "vulkan"))]
            device_info,
            deferred_dropper: DeferredDropper::new(3),
            destroyed: AtomicBool::new(false),

            #[cfg(any(feature = "vulkan"))]
            platform_device_context,

            #[cfg(debug_assertions)]
            #[cfg(feature = "track-device-contexts")]
            all_contexts: Mutex::new(all_contexts),

            #[cfg(debug_assertions)]
            #[cfg(feature = "track-device-contexts")]
            next_create_index: AtomicU64::new(1),
        })
    }
}

#[derive(Debug)]
pub struct DeviceContext {
    pub(crate) inner: Arc<DeviceContextInner>,
    #[cfg(debug_assertions)]
    #[cfg(feature = "track-device-contexts")]
    pub(super) create_index: u64,
}

impl Clone for DeviceContext {
    fn clone(&self) -> Self {
        #[cfg(debug_assertions)]
        #[cfg(feature = "track-device-contexts")]
        let create_index = {
            let create_index = self.inner.next_create_index.fetch_add(1, Ordering::Relaxed);

            #[cfg(feature = "track-device-contexts")]
            {
                let create_backtrace = backtrace::Backtrace::new_unresolved();
                self.inner
                    .as_ref()
                    .all_contexts
                    .lock()
                    .unwrap()
                    .insert(create_index, create_backtrace);
            }

            log::trace!("Cloned VulkanDeviceContext create_index {}", create_index);
            create_index
        };
        Self {
            inner: self.inner.clone(),
            #[cfg(debug_assertions)]
            #[cfg(feature = "track-device-contexts")]
            create_index,
        }
    }
}

impl Drop for DeviceContext {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        #[cfg(feature = "track-device-contexts")]
        {
            self.inner
                .all_contexts
                .lock()
                .unwrap()
                .remove(&self.create_index);
        }
    }
}

impl DeviceContext {
    pub fn new(
        #[cfg(not(any(feature = "vulkan")))] instance: &Instance,
        #[cfg(any(feature = "vulkan"))] instance: &Instance<'_>,
        api_def: &ApiDef,
    ) -> GfxResult<Self> {
        let inner = Arc::new(DeviceContextInner::new(
            instance,
            api_def.windowing_mode,
            api_def.video_mode,
        )?);

        Ok(Self {
            inner,
            #[cfg(debug_assertions)]
            #[cfg(feature = "track-device-contexts")]
            create_index: 0,
        })
    }

    #[cfg(any(feature = "vulkan"))]
    pub fn device_info(&self) -> &DeviceInfo {
        &self.inner.device_info
    }

    pub fn deferred_dropper(&self) -> &DeferredDropper {
        &self.inner.deferred_dropper
    }

    #[cfg(feature = "vulkan")]
    pub(crate) fn platform_device_context(&self) -> &VulkanDeviceContext {
        &self.inner.platform_device_context
    }

    #[cfg(feature = "vulkan")]
    pub fn platform_device(&self) -> &ash::Device {
        self.inner.platform_device_context.device()
    }

    pub fn create_queue(&self, queue_type: QueueType) -> GfxResult<Queue> {
        Queue::new(self, queue_type)
    }

    pub fn create_fence(&self) -> GfxResult<Fence> {
        Fence::new(self)
    }

    pub fn create_semaphore(&self) -> GfxResult<Semaphore> {
        Semaphore::new(self)
    }

    pub fn create_swapchain(
        &self,
        raw_window_handle: &dyn HasRawWindowHandle,
        swapchain_def: &SwapchainDef,
    ) -> GfxResult<Swapchain> {
        Swapchain::new(self, raw_window_handle, swapchain_def)
    }

    pub fn wait_for_fences(&self, fences: &[&Fence]) -> GfxResult<()> {
        Fence::wait_for_fences(self, fences)
    }

    pub fn create_sampler(&self, sampler_def: &SamplerDef) -> GfxResult<Sampler> {
        Sampler::new(self, sampler_def)
    }

    pub fn create_texture(&self, texture_def: &TextureDef) -> GfxResult<Texture> {
        Texture::new(self, texture_def)
    }

    pub fn create_buffer(&self, buffer_def: &BufferDef) -> GfxResult<Buffer> {
        Buffer::new(self, buffer_def)
    }

    pub fn create_shader(
        &self,
        stages: Vec<ShaderStageDef>,
        pipeline_reflection: &PipelineReflection,
    ) -> GfxResult<Shader> {
        Shader::new(self, stages, pipeline_reflection)
    }

    pub fn create_descriptorset_layout(
        &self,
        descriptorset_layout_def: &DescriptorSetLayoutDef,
    ) -> GfxResult<DescriptorSetLayout> {
        DescriptorSetLayout::new(self, descriptorset_layout_def)
    }

    pub fn create_root_signature(
        &self,
        root_signature_def: &RootSignatureDef,
    ) -> GfxResult<RootSignature> {
        RootSignature::new(self, root_signature_def)
    }

    pub fn create_descriptor_heap(
        &self,
        descriptor_heap_def: &DescriptorHeapDef,
    ) -> GfxResult<DescriptorHeap> {
        DescriptorHeap::new(self, descriptor_heap_def)
    }

    pub fn create_graphics_pipeline(
        &self,
        graphics_pipeline_def: &GraphicsPipelineDef<'_>,
    ) -> GfxResult<Pipeline> {
        Pipeline::new_graphics_pipeline(self, graphics_pipeline_def)
    }

    pub fn create_compute_pipeline(
        &self,
        compute_pipeline_def: &ComputePipelineDef<'_>,
    ) -> GfxResult<Pipeline> {
        Pipeline::new_compute_pipeline(self, compute_pipeline_def)
    }

    pub fn create_shader_module(&self, data: ShaderModuleDef<'_>) -> GfxResult<ShaderModule> {
        ShaderModule::new(self, data)
    }

    pub fn free_gpu_memory(&self) -> GfxResult<()> {
        self.inner.deferred_dropper.flush();
        Ok(())
    }
}
