use std::ffi::CStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use ash::extensions::khr;
use ash::vk;
use fnv::FnvHashMap;
use raw_window_handle::HasRawWindowHandle;

use super::internal::{
    DeviceVulkanResourceCache, VkInstance, VkQueueAllocationStrategy, VkQueueAllocatorSet,
    VkQueueRequirements, VulkanRenderpass, VulkanRenderpassDef,
};
use super::{
    VulkanApi, VulkanBuffer, VulkanDescriptorHeap, VulkanDescriptorSetArray,
    VulkanDescriptorSetLayout, VulkanFence, VulkanPipeline, VulkanQueue, VulkanRootSignature,
    VulkanSampler, VulkanSemaphore, VulkanShader, VulkanShaderModule, VulkanSwapchain,
    VulkanTexture,
};
use crate::backends::deferred_drop::DeferredDropper;
use crate::vulkan::check_extensions_availability;
use crate::{ApiDef, BufferDef, ComputePipelineDef, DescriptorHeapDef, DescriptorSetArrayDef, DescriptorSetLayoutDef, DeviceContext, DeviceInfo, ExtensionMode, Fence, GfxResult, GraphicsPipelineDef, PipelineReflection, QueueType, RootSignatureDef, SamplerDef, ShaderModuleDef, ShaderStageDef, SwapchainDef, TextureDef};

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

impl PhysicalDeviceType {
    /// Convert to `vk::PhysicalDeviceType`
    pub fn to_vk(self) -> vk::PhysicalDeviceType {
        match self {
            PhysicalDeviceType::Other => vk::PhysicalDeviceType::OTHER,
            PhysicalDeviceType::IntegratedGpu => vk::PhysicalDeviceType::INTEGRATED_GPU,
            PhysicalDeviceType::DiscreteGpu => vk::PhysicalDeviceType::DISCRETE_GPU,
            PhysicalDeviceType::VirtualGpu => vk::PhysicalDeviceType::VIRTUAL_GPU,
            PhysicalDeviceType::Cpu => vk::PhysicalDeviceType::CPU,
        }
    }
}

#[derive(Clone)]
pub struct PhysicalDeviceInfo {
    pub score: i32,
    pub queue_family_indices: VkQueueFamilyIndices,
    pub properties: vk::PhysicalDeviceProperties,
    pub features: vk::PhysicalDeviceFeatures,
    pub extension_properties: Vec<ash::vk::ExtensionProperties>,
    pub all_queue_families: Vec<ash::vk::QueueFamilyProperties>,
    pub required_extensions: Vec<&'static CStr>,
}

#[derive(Default, Clone, Debug)]
pub struct VkQueueFamilyIndices {
    pub graphics_queue_family_index: u32,
    pub compute_queue_family_index: u32,
    pub transfer_queue_family_index: u32,
    pub decode_queue_family_index: Option<u32>,
    pub encode_queue_family_index: Option<u32>,
}

pub(super) struct VulkanDeviceContextInner {
    pub(crate) resource_cache: DeviceVulkanResourceCache,
    // pub(crate) descriptor_heap: VulkanDescriptorHeap,
    device_info: DeviceInfo,
    queue_allocator: VkQueueAllocatorSet,

    // If we need a dedicated present queue, we share a single queue across all swapchains. This
    // lock ensures that the present operations for those swapchains do not occur concurrently
    dedicated_present_queue_lock: Mutex<()>,

    device: ash::Device,
    allocator: vk_mem::Allocator,
    pub(crate) deferred_dropper: DeferredDropper,
    destroyed: AtomicBool,
    entry: Arc<ash::Entry>,
    instance: ash::Instance,
    physical_device: vk::PhysicalDevice,
    physical_device_info: PhysicalDeviceInfo,

    #[cfg(debug_assertions)]
    #[cfg(feature = "track-device-contexts")]
    next_create_index: AtomicU64,

    #[cfg(debug_assertions)]
    #[cfg(feature = "track-device-contexts")]
    all_contexts: Mutex<fnv::FnvHashMap<u64, backtrace::Backtrace>>,
}

impl Drop for VulkanDeviceContextInner {
    fn drop(&mut self) {
        if !self.destroyed.swap(true, Ordering::AcqRel) {
            unsafe {
                log::trace!("destroying device");
                self.deferred_dropper.destroy();
                self.allocator.destroy();
                self.device.destroy_device(None);
                //self.surface_loader.destroy_surface(self.surface, None);
                log::trace!("destroyed device");
            }
        }
    }
}

impl VulkanDeviceContextInner {
    pub fn new(
        instance: &VkInstance,
        windowing_mode: ExtensionMode,
        video_mode: ExtensionMode,
    ) -> GfxResult<Self> {
        // Pick a physical device
        let (physical_device, physical_device_info) =
            choose_physical_device(&instance.instance, windowing_mode, video_mode)?;

        //TODO: Don't hardcode queue counts
        let queue_requirements = VkQueueRequirements::determine_required_queue_counts(
            &physical_device_info.queue_family_indices,
            &physical_device_info.all_queue_families,
            VkQueueAllocationStrategy::ShareFirstQueueInFamily,
            VkQueueAllocationStrategy::ShareFirstQueueInFamily,
            VkQueueAllocationStrategy::ShareFirstQueueInFamily,
            VkQueueAllocationStrategy::ShareFirstQueueInFamily,
            VkQueueAllocationStrategy::ShareFirstQueueInFamily,
        );

        // Create a logical device
        let logical_device = create_logical_device(
            &instance.instance,
            physical_device,
            &physical_device_info,
            &queue_requirements,
        )?;

        let queue_allocator = VkQueueAllocatorSet::new(
            &logical_device,
            &physical_device_info.all_queue_families,
            queue_requirements,
        );

        let allocator_create_info = vk_mem::AllocatorCreateInfo {
            physical_device,
            device: logical_device.clone(),
            instance: instance.instance.clone(),
            flags: vk_mem::AllocatorCreateFlags::default(),
            preferred_large_heap_block_size: Default::default(),
            frame_in_use_count: 0, // Not using CAN_BECOME_LOST, so this is not needed
            heap_size_limits: Option::default(),
        };

        let allocator = vk_mem::Allocator::new(&allocator_create_info)?;

        let limits = &physical_device_info.properties.limits;

        let device_info = DeviceInfo {
            supports_multithreaded_usage: true,
            min_uniform_buffer_offset_alignment: limits.min_uniform_buffer_offset_alignment as u32,
            min_storage_buffer_offset_alignment: limits.min_storage_buffer_offset_alignment as u32,
            upload_buffer_texture_alignment: limits.optimal_buffer_copy_offset_alignment as u32,
            upload_buffer_texture_row_alignment: limits.optimal_buffer_copy_row_pitch_alignment
                as u32,
            supports_clamp_to_border_color: true,
            max_vertex_attribute_count: limits.max_vertex_input_attributes,
        };

        let resource_cache = DeviceVulkanResourceCache::default();
        // let descriptor_heap = VulkanDescriptorHeap::new_old(&logical_device,
        //     &DescriptorHeapDef {
        //         transient: false,
        //         max_descriptor_sets: 8192,
        //         sampler_count: 1024,
        //         constant_buffer_count: 8192,
        //         buffer_count: 8192,
        //         rw_buffer_count: 1024,
        //         texture_count: 8192,
        //         rw_texture_count: 1024,
        //     }
        // )?;

        #[cfg(debug_assertions)]
        #[cfg(feature = "track-device-contexts")]
        let all_contexts = {
            let create_backtrace = backtrace::Backtrace::new_unresolved();
            let mut all_contexts = fnv::FnvHashMap::<u64, backtrace::Backtrace>::default();
            all_contexts.insert(0, create_backtrace);
            all_contexts
        };

        Ok(Self {
            resource_cache,
            device_info,
            queue_allocator,
            dedicated_present_queue_lock: Mutex::default(),
            entry: instance.entry.clone(),
            instance: instance.instance.clone(),
            physical_device,
            physical_device_info,
            device: logical_device,
            allocator,
            deferred_dropper: DeferredDropper::new(3),
            destroyed: AtomicBool::new(false),

            #[cfg(debug_assertions)]
            #[cfg(feature = "track-device-contexts")]
            all_contexts: Mutex::new(all_contexts),

            #[cfg(debug_assertions)]
            #[cfg(feature = "track-device-contexts")]
            next_create_index: AtomicU64::new(1),
        })
    }
}

pub struct VulkanDeviceContext {
    pub(super) inner: Arc<VulkanDeviceContextInner>,
    #[cfg(debug_assertions)]
    #[cfg(feature = "track-device-contexts")]
    pub(super) create_index: u64,
}

impl std::fmt::Debug for VulkanDeviceContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VulkanDeviceContext")
            .field("handle", &self.device().handle())
            .finish()
    }
}

impl Clone for VulkanDeviceContext {
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

impl Drop for VulkanDeviceContext {
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

impl VulkanDeviceContext {
    pub(crate) fn resource_cache(&self) -> &DeviceVulkanResourceCache {
        &self.inner.resource_cache
    }

    pub fn entry(&self) -> &ash::Entry {
        &*self.inner.entry
    }

    pub fn instance(&self) -> &ash::Instance {
        &self.inner.instance
    }

    pub fn device(&self) -> &ash::Device {
        &self.inner.device
    }

    pub fn physical_device(&self) -> vk::PhysicalDevice {
        self.inner.physical_device
    }

    pub fn physical_device_info(&self) -> &PhysicalDeviceInfo {
        &self.inner.physical_device_info
    }

    pub fn limits(&self) -> &vk::PhysicalDeviceLimits {
        &self.physical_device_info().properties.limits
    }

    pub fn allocator(&self) -> &vk_mem::Allocator {
        &self.inner.allocator
    }

    pub fn queue_allocator(&self) -> &VkQueueAllocatorSet {
        &self.inner.queue_allocator
    }

    pub fn queue_family_indices(&self) -> &VkQueueFamilyIndices {
        &self.inner.physical_device_info.queue_family_indices
    }

    pub fn dedicated_present_queue_lock(&self) -> &Mutex<()> {
        &self.inner.dedicated_present_queue_lock
    }

    pub fn deferred_dropper(&self) -> &DeferredDropper {
        &self.inner.deferred_dropper
    }

    pub(crate) fn create_renderpass(
        &self,
        renderpass_def: &VulkanRenderpassDef,
    ) -> GfxResult<VulkanRenderpass> {
        VulkanRenderpass::new(self, renderpass_def)
    }

    pub fn new(instance: &VkInstance, api_def: &ApiDef) -> GfxResult<Self> {
        let inner = Arc::new(VulkanDeviceContextInner::new(
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
}

impl DeviceContext<VulkanApi> for VulkanDeviceContext {
    fn device_info(&self) -> &DeviceInfo {
        &self.inner.device_info
    }

    fn create_queue(&self, queue_type: QueueType) -> GfxResult<VulkanQueue> {
        VulkanQueue::new(self, queue_type)
    }

    fn create_fence(&self) -> GfxResult<VulkanFence> {
        VulkanFence::new(self)
    }

    fn create_semaphore(&self) -> GfxResult<VulkanSemaphore> {
        VulkanSemaphore::new(self)
    }

    fn create_swapchain(
        &self,
        raw_window_handle: &dyn HasRawWindowHandle,
        swapchain_def: &SwapchainDef,
    ) -> GfxResult<VulkanSwapchain> {
        VulkanSwapchain::new(self, raw_window_handle, swapchain_def)
    }

    fn wait_for_fences(&self, fences: &[&VulkanFence]) -> GfxResult<()> {
        VulkanFence::wait_for_fences(self, fences)
    }

    fn create_sampler(&self, sampler_def: &SamplerDef) -> GfxResult<VulkanSampler> {
        VulkanSampler::new(self, sampler_def)
    }

    fn create_texture(&self, texture_def: &TextureDef) -> GfxResult<VulkanTexture> {
        VulkanTexture::new(self, texture_def)
    }

    fn create_buffer(&self, buffer_def: &BufferDef) -> GfxResult<VulkanBuffer> {
        VulkanBuffer::new(self, buffer_def)
    }

    fn create_shader(
        &self,
        stages: Vec<ShaderStageDef<VulkanApi>>,
        pipeline_reflection: &PipelineReflection,
    ) -> GfxResult<VulkanShader> {
        VulkanShader::new(self, stages, pipeline_reflection)
    }

    fn create_descriptorset_layout(
        &self,
        descriptorset_layout_def: &DescriptorSetLayoutDef,
    ) -> GfxResult<VulkanDescriptorSetLayout> {
        VulkanDescriptorSetLayout::new(self, descriptorset_layout_def)
    }

    fn create_root_signature(
        &self,
        root_signature_def: &RootSignatureDef<VulkanApi>,
    ) -> GfxResult<VulkanRootSignature> {
        VulkanRootSignature::new(self, root_signature_def)
    }

    fn create_descriptor_set_array(
        &self,
        descriptor_heap: VulkanDescriptorHeap,
        descriptor_set_array_def: &DescriptorSetArrayDef<'_, VulkanApi>,
    ) -> GfxResult<VulkanDescriptorSetArray> {
        VulkanDescriptorSetArray::new(self, descriptor_heap, descriptor_set_array_def)
    }

    fn create_descriptor_heap(
        &self,
        descriptor_heap_def: &DescriptorHeapDef,
    ) -> GfxResult<VulkanDescriptorHeap> {
        VulkanDescriptorHeap::new(self, descriptor_heap_def)
    }

    fn create_graphics_pipeline(
        &self,
        graphics_pipeline_def: &GraphicsPipelineDef<'_, VulkanApi>,
    ) -> GfxResult<VulkanPipeline> {
        VulkanPipeline::new_graphics_pipeline(self, graphics_pipeline_def)
    }

    fn create_compute_pipeline(
        &self,
        compute_pipeline_def: &ComputePipelineDef<'_, VulkanApi>,
    ) -> GfxResult<VulkanPipeline> {
        VulkanPipeline::new_compute_pipeline(self, compute_pipeline_def)
    }

    fn create_shader_module(&self, data: ShaderModuleDef<'_>) -> GfxResult<VulkanShaderModule> {
        VulkanShaderModule::new(self, data)
    }

    fn free_gpu_memory(&self) -> GfxResult<()> {
        self.inner.deferred_dropper.flush();
        Ok(())
    }
}

fn choose_physical_device(
    instance: &ash::Instance,
    windowing_mode: ExtensionMode,
    video_mode: ExtensionMode,
) -> GfxResult<(ash::vk::PhysicalDevice, PhysicalDeviceInfo)> {
    let physical_devices = unsafe { instance.enumerate_physical_devices()? };

    if physical_devices.is_empty() {
        panic!("Could not find a physical device");
    }

    let mut best_physical_device = None;
    let mut best_physical_device_info = None;
    let mut best_physical_device_score = -1;
    // let mut best_physical_device_queue_family_indices = None;
    for physical_device in physical_devices {
        let result =
            query_physical_device_info(instance, physical_device, windowing_mode, video_mode);

        if let Some(physical_device_info) = result? {
            if physical_device_info.score > best_physical_device_score {
                best_physical_device = Some(physical_device);
                best_physical_device_score = physical_device_info.score;
                best_physical_device_info = Some(physical_device_info);
            }
        }
    }

    //TODO: Return an error
    let physical_device = best_physical_device.expect("Could not find suitable device");
    let physical_device_info = best_physical_device_info.unwrap();

    Ok((physical_device, physical_device_info))
}

fn vk_version_to_string(version: u32) -> String {
    format!(
        "{}.{}.{}",
        vk::api_version_major(version),
        vk::api_version_minor(version),
        vk::api_version_patch(version)
    )
}

fn query_physical_device_info(
    instance: &ash::Instance,
    device: ash::vk::PhysicalDevice,
    windowing_mode: ExtensionMode,
    video_mode: ExtensionMode,
) -> GfxResult<Option<PhysicalDeviceInfo>> {
    let physical_device_type_priority = [
        PhysicalDeviceType::DiscreteGpu,
        PhysicalDeviceType::IntegratedGpu,
    ];
    log::info!(
        "Preferred device types: {:?}",
        physical_device_type_priority
    );
    let properties: ash::vk::PhysicalDeviceProperties =
        unsafe { instance.get_physical_device_properties(device) };
    let device_name = unsafe {
        CStr::from_ptr(properties.device_name.as_ptr())
            .to_str()
            .unwrap()
            .to_string()
    };

    let extensions: Vec<ash::vk::ExtensionProperties> =
        unsafe { instance.enumerate_device_extension_properties(device)? };
    log::debug!("Available device extensions: {:#?}", extensions);
    let mut required_extensions = vec![];
    match windowing_mode {
        ExtensionMode::Disabled => {}
        ExtensionMode::EnabledIfAvailable | ExtensionMode::Enabled => {
            if check_extensions_availability(&[khr::Swapchain::name()], &extensions) {
                required_extensions.push(khr::Swapchain::name());
            } else if windowing_mode == ExtensionMode::EnabledIfAvailable {
                log::info!("Could not find the swapchain extension. Check that the proper drivers are installed.");
            } else {
                log::warn!("Could not find the swapchain extension. Check that the proper drivers are installed.");
                return Ok(None);
            }
        }
    };
    match video_mode {
        ExtensionMode::Disabled => {}
        ExtensionMode::EnabledIfAvailable | ExtensionMode::Enabled => {
            let video_extensions = [
                vk::KhrVideoQueueFn::name(),
                vk::KhrVideoDecodeQueueFn::name(),
                vk::KhrVideoEncodeQueueFn::name(),
                khr::Synchronization2::name(),
            ];
            if check_extensions_availability(&video_extensions, &extensions) {
                required_extensions.extend_from_slice(&video_extensions);
            } else if video_mode == ExtensionMode::EnabledIfAvailable {
                log::info!("Could not find the Vulkan video extensions. Check that the proper drivers are installed.");
            } else {
                log::warn!("Could not find the Vulkan video extensions. Check that the proper drivers are installed.");
                return Ok(None);
            }
        }
    };

    let features: vk::PhysicalDeviceFeatures =
        unsafe { instance.get_physical_device_features(device) };
    let all_queue_families: Vec<ash::vk::QueueFamilyProperties> =
        unsafe { instance.get_physical_device_queue_family_properties(device) };

    if let Some(queue_family_indices) = find_queue_families(&all_queue_families) {
        // Determine the index of the device_type within physical_device_type_priority
        let index = physical_device_type_priority
            .iter()
            .map(|x| x.to_vk())
            .position(|x| x == properties.device_type);

        // Convert it to a score
        let device_type_rank: i32 = index
            .map_or(0, |index| physical_device_type_priority.len() - index)
            .try_into()
            .unwrap();
        let extensions_rank: i32 = required_extensions.len().try_into().unwrap();
        let score = device_type_rank * 1000 + extensions_rank * 10;

        log::info!(
            "Found suitable device '{}' API: {} DriverVersion: {} Score = {}",
            device_name,
            vk_version_to_string(properties.api_version),
            vk_version_to_string(properties.driver_version),
            score
        );

        let result = PhysicalDeviceInfo {
            score,
            queue_family_indices,
            properties,
            extension_properties: extensions,
            features,
            all_queue_families,
            required_extensions,
        };

        log::trace!("{:#?}", properties);
        Ok(Some(result))
    } else {
        log::info!(
            "Found unsuitable device '{}' API: {} DriverVersion: {} could not find queue families",
            device_name,
            vk_version_to_string(properties.api_version),
            vk_version_to_string(properties.driver_version)
        );
        log::trace!("{:#?}", properties);
        Ok(None)
    }
}

//TODO: Could improve this by looking at vendor/device ID, VRAM size, supported feature set, etc.
#[allow(clippy::too_many_lines)]
fn find_queue_families(
    all_queue_families: &[ash::vk::QueueFamilyProperties],
) -> Option<VkQueueFamilyIndices> {
    let mut graphics_queue_family_index = None;
    let mut compute_queue_family_index = None;
    let mut transfer_queue_family_index = None;
    let mut decode_queue_family_index = None;
    let mut encode_queue_family_index = None;

    log::info!("Available queue families:");
    for (queue_family_index, queue_family) in all_queue_families.iter().enumerate() {
        log::info!("Queue Family {}", queue_family_index);
        log::info!("{:#?}", queue_family);
    }

    //
    // Find the first queue family that supports graphics and use it for graphics
    //
    for (queue_family_index, queue_family) in all_queue_families.iter().enumerate() {
        let queue_family_index = queue_family_index as u32;
        let supports_graphics = queue_family.queue_flags & ash::vk::QueueFlags::GRAPHICS
            == ash::vk::QueueFlags::GRAPHICS;

        if supports_graphics {
            graphics_queue_family_index = Some(queue_family_index);
            break;
        }
    }

    //
    // Find a compute queue family in the following order of preference:
    // - Doesn't support graphics
    // - Supports graphics but hasn't already been claimed by graphics
    // - Fallback to using the graphics queue family as it's guaranteed to support compute
    //
    for (queue_family_index, queue_family) in all_queue_families.iter().enumerate() {
        let queue_family_index = queue_family_index as u32;
        let supports_graphics = queue_family.queue_flags & ash::vk::QueueFlags::GRAPHICS
            == ash::vk::QueueFlags::GRAPHICS;
        let supports_compute =
            queue_family.queue_flags & ash::vk::QueueFlags::COMPUTE == ash::vk::QueueFlags::COMPUTE;

        if !supports_graphics && supports_compute {
            // Ideally we want to find a dedicated compute queue (i.e. doesn't support graphics)
            compute_queue_family_index = Some(queue_family_index);
            break;
        } else if supports_compute
            && compute_queue_family_index.is_none()
            && Some(queue_family_index) != graphics_queue_family_index
        {
            // Otherwise accept the first queue that supports compute that is NOT the graphics queue
            compute_queue_family_index = Some(queue_family_index);
        }
    }

    // If we didn't find a compute queue family != graphics queue family, settle for using the
    // graphics queue family. It's guaranteed to support compute.
    if compute_queue_family_index.is_none() {
        compute_queue_family_index = graphics_queue_family_index;
    }

    //
    // Find a transfer queue family in the following order of preference:
    // - Doesn't support graphics or compute
    // - Supports graphics but hasn't already been claimed by compute or graphics
    // - Fallback to using the graphics queue family as it's guaranteed to support transfers
    //
    for (queue_family_index, queue_family) in all_queue_families.iter().enumerate() {
        let queue_family_index = queue_family_index as u32;
        let supports_graphics = queue_family.queue_flags & ash::vk::QueueFlags::GRAPHICS
            == ash::vk::QueueFlags::GRAPHICS;
        let supports_compute =
            queue_family.queue_flags & ash::vk::QueueFlags::COMPUTE == ash::vk::QueueFlags::COMPUTE;
        let supports_transfer = queue_family.queue_flags & ash::vk::QueueFlags::TRANSFER
            == ash::vk::QueueFlags::TRANSFER;

        if !supports_graphics && !supports_compute && supports_transfer {
            // Ideally we want to find a dedicated transfer queue
            transfer_queue_family_index = Some(queue_family_index);
            break;
        } else if supports_transfer
            && transfer_queue_family_index.is_none()
            && Some(queue_family_index) != graphics_queue_family_index
            && Some(queue_family_index) != compute_queue_family_index
        {
            // Otherwise accept the first queue that supports transfers that is NOT the graphics queue or compute queue
            transfer_queue_family_index = Some(queue_family_index);
        }
    }

    // If we didn't find a transfer queue family != graphics queue family, settle for using the
    // graphics queue family. It's guaranteed to support transfer.
    if transfer_queue_family_index.is_none() {
        transfer_queue_family_index = graphics_queue_family_index;
    }

    //
    // Find a decode queue family in the following order of preference:
    // - Doesn't support graphics, compute, encode
    // - Supports decode
    //
    for (queue_family_index, queue_family) in all_queue_families.iter().enumerate() {
        let queue_family_index = queue_family_index as u32;
        let supports_graphics = queue_family.queue_flags & ash::vk::QueueFlags::GRAPHICS
            == ash::vk::QueueFlags::GRAPHICS;
        let supports_compute =
            queue_family.queue_flags & ash::vk::QueueFlags::COMPUTE == ash::vk::QueueFlags::COMPUTE;
        let supports_decode = queue_family.queue_flags & ash::vk::QueueFlags::VIDEO_DECODE_KHR
            == ash::vk::QueueFlags::VIDEO_DECODE_KHR;
        let supports_encode = queue_family.queue_flags & ash::vk::QueueFlags::VIDEO_ENCODE_KHR
            == ash::vk::QueueFlags::VIDEO_ENCODE_KHR;

        if !supports_graphics && !supports_compute && !supports_encode && supports_decode {
            // Ideally we want to find a dedicated transfer queue
            decode_queue_family_index = Some(queue_family_index);
            break;
        } else if supports_decode && decode_queue_family_index.is_none() {
            // Otherwise accept the first queue that supports transfers that is NOT the graphics queue or compute queue
            decode_queue_family_index = Some(queue_family_index);
        }
    }

    //
    // Find a encode queue family in the following order of preference:
    // - Doesn't support graphics, compute, decode
    // - Supports encode
    //
    for (queue_family_index, queue_family) in all_queue_families.iter().enumerate() {
        let queue_family_index = queue_family_index as u32;
        let supports_graphics = queue_family.queue_flags & ash::vk::QueueFlags::GRAPHICS
            == ash::vk::QueueFlags::GRAPHICS;
        let supports_compute =
            queue_family.queue_flags & ash::vk::QueueFlags::COMPUTE == ash::vk::QueueFlags::COMPUTE;
        let supports_decode = queue_family.queue_flags & ash::vk::QueueFlags::VIDEO_DECODE_KHR
            == ash::vk::QueueFlags::VIDEO_DECODE_KHR;
        let supports_encode = queue_family.queue_flags & ash::vk::QueueFlags::VIDEO_ENCODE_KHR
            == ash::vk::QueueFlags::VIDEO_ENCODE_KHR;

        if !supports_graphics && !supports_compute && !supports_decode && supports_encode {
            // Ideally we want to find a dedicated transfer queue
            encode_queue_family_index = Some(queue_family_index);
            break;
        } else if supports_decode && encode_queue_family_index.is_none() {
            // Otherwise accept the first queue that supports transfers that is NOT the graphics queue or compute queue
            encode_queue_family_index = Some(queue_family_index);
        }
    }

    log::info!(
        "Graphics QF: {:?}  Compute QF: {:?}  Transfer QF: {:?}  Decode QF: {:?}  Encode QF: {:?}",
        graphics_queue_family_index,
        compute_queue_family_index,
        transfer_queue_family_index,
        decode_queue_family_index,
        encode_queue_family_index,
    );

    if let (
        Some(graphics_queue_family_index),
        Some(compute_queue_family_index),
        Some(transfer_queue_family_index),
    ) = (
        graphics_queue_family_index,
        compute_queue_family_index,
        transfer_queue_family_index,
    ) {
        Some(VkQueueFamilyIndices {
            graphics_queue_family_index,
            compute_queue_family_index,
            transfer_queue_family_index,
            decode_queue_family_index,
            encode_queue_family_index,
        })
    } else {
        None
    }
}

fn create_logical_device(
    instance: &ash::Instance,
    physical_device: ash::vk::PhysicalDevice,
    physical_device_info: &PhysicalDeviceInfo,
    queue_requirements: &VkQueueRequirements,
) -> GfxResult<ash::Device> {
    //TODO: Ideally we would set up validation layers for the logical device too.
    let mut device_extension_names: Vec<_> = physical_device_info
        .required_extensions
        .iter()
        .map(|name| name.as_ptr())
        .collect();

    // Add VK_KHR_portability_subset if the extension exists (this is mandated by spec)
    for extension in &physical_device_info.extension_properties {
        let extension_name = unsafe { CStr::from_ptr(extension.extension_name.as_ptr()) };

        if extension_name == vk::KhrPortabilitySubsetFn::name() {
            device_extension_names.push(vk::KhrPortabilitySubsetFn::name().as_ptr());
            break;
        }
    }

    // Features enabled here by default are supported very widely (only unsupported devices on
    // vulkan.gpuinfo.org are SwiftShader, a software renderer.
    let features = vk::PhysicalDeviceFeatures::builder()
        .sampler_anisotropy(true)
        .sample_rate_shading(true)
        // Used for debug drawing lines/points
        .fill_mode_non_solid(true);

    let mut queue_families_to_create = FnvHashMap::default();
    for (&queue_family_index, &count) in &queue_requirements.queue_counts {
        queue_families_to_create.insert(queue_family_index, vec![1.0_f32; count as usize]);
    }

    let queue_infos: Vec<_> = queue_families_to_create
        .iter()
        .map(|(&queue_family_index, priorities)| {
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index)
                .queue_priorities(priorities)
                .build()
        })
        .collect();

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(&device_extension_names)
        .enabled_features(&features);

    let device: ash::Device =
        unsafe { instance.create_device(physical_device, &device_create_info, None)? };

    Ok(device)
}
