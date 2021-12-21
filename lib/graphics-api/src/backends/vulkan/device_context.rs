use std::ffi::CStr;
use std::sync::{Arc, Mutex};

use ash::extensions::khr;
use ash::vk;
use fnv::FnvHashMap;

use super::{
    DeviceVulkanResourceCache, VkInstance, VkQueueAllocationStrategy, VkQueueAllocatorSet,
    VkQueueRequirements, VulkanRenderpass, VulkanRenderpassDef,
};
use crate::backends::vulkan::check_extensions_availability;
use crate::{DeviceContext, DeviceInfo, ExtensionMode, GfxResult, PhysicalDeviceType};

impl PhysicalDeviceType {
    /// Convert to `vk::PhysicalDeviceType`
    pub(crate) fn to_vk(self) -> vk::PhysicalDeviceType {
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
pub(crate) struct PhysicalDeviceInfo {
    pub(crate) score: i32,
    pub(crate) queue_family_indices: VkQueueFamilyIndices,
    pub(crate) properties: vk::PhysicalDeviceProperties,
    pub(crate) _features: vk::PhysicalDeviceFeatures,
    pub(crate) extension_properties: Vec<ash::vk::ExtensionProperties>,
    pub(crate) all_queue_families: Vec<ash::vk::QueueFamilyProperties>,
    pub(crate) required_extensions: Vec<&'static CStr>,
}

#[derive(Default, Clone, Debug)]
pub(crate) struct VkQueueFamilyIndices {
    pub(crate) graphics_queue_family_index: u32,
    pub(crate) compute_queue_family_index: u32,
    pub(crate) transfer_queue_family_index: u32,
    pub(crate) decode_queue_family_index: Option<u32>,
    pub(crate) encode_queue_family_index: Option<u32>,
}

pub(crate) struct VulkanDeviceContext {
    resource_cache: DeviceVulkanResourceCache,
    queue_allocator: VkQueueAllocatorSet,

    // If we need a dedicated present queue, we share a single queue across all swapchains. This
    // lock ensures that the present operations for those swapchains do not occur concurrently
    dedicated_present_queue_lock: Mutex<()>,

    vk_device: ash::Device,
    vk_allocator: vk_mem::Allocator,
    entry: Arc<ash::Entry>,
    instance: ash::Instance,
    vk_physical_device: vk::PhysicalDevice,
    physical_device_info: PhysicalDeviceInfo,

    #[cfg(debug_assertions)]
    #[cfg(feature = "track-device-contexts")]
    next_create_index: AtomicU64,

    #[cfg(debug_assertions)]
    #[cfg(feature = "track-device-contexts")]
    all_contexts: Mutex<fnv::FnvHashMap<u64, backtrace::Backtrace>>,
}

impl VulkanDeviceContext {
    pub(crate) fn new(
        instance: &VkInstance,
        windowing_mode: ExtensionMode,
        video_mode: ExtensionMode,
    ) -> GfxResult<(Self, DeviceInfo)> {
        // Pick a physical device
        let (vk_physical_device, physical_device_info) =
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
        let vk_logical_device = create_logical_device(
            &instance.instance,
            vk_physical_device,
            &physical_device_info,
            &queue_requirements,
        )?;

        let queue_allocator = VkQueueAllocatorSet::new(
            &vk_logical_device,
            &physical_device_info.all_queue_families,
            queue_requirements,
        );

        let allocator_create_info = vk_mem::AllocatorCreateInfo {
            physical_device: vk_physical_device,
            device: vk_logical_device.clone(),
            instance: instance.instance.clone(),
            flags: vk_mem::AllocatorCreateFlags::default(),
            preferred_large_heap_block_size: Default::default(),
            frame_in_use_count: 0, // Not using CAN_BECOME_LOST, so this is not needed
            heap_size_limits: Option::default(),
        };

        let vk_allocator = vk_mem::Allocator::new(&allocator_create_info)?;

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

        #[cfg(debug_assertions)]
        #[cfg(feature = "track-device-contexts")]
        let all_contexts = {
            let create_backtrace = backtrace::Backtrace::new_unresolved();
            let mut all_contexts = fnv::FnvHashMap::<u64, backtrace::Backtrace>::default();
            all_contexts.insert(0, create_backtrace);
            all_contexts
        };

        Ok((
            Self {
                resource_cache,
                queue_allocator,
                dedicated_present_queue_lock: Mutex::default(),
                entry: instance.entry.clone(),
                instance: instance.instance.clone(),
                vk_physical_device,
                physical_device_info,
                vk_device: vk_logical_device,
                vk_allocator,

                #[cfg(debug_assertions)]
                #[cfg(feature = "track-device-contexts")]
                all_contexts: Mutex::new(all_contexts),

                #[cfg(debug_assertions)]
                #[cfg(feature = "track-device-contexts")]
                next_create_index: AtomicU64::new(1),
            },
            device_info,
        ))
    }

    pub(crate) fn destroy(&mut self) {
        unsafe {
            self.vk_allocator.destroy();
            self.vk_device.destroy_device(None);
        }
    }
}

impl DeviceContext {
    pub(crate) fn resource_cache(&self) -> &DeviceVulkanResourceCache {
        &self.inner.platform_device_context.resource_cache
    }

    pub(crate) fn vk_entry(&self) -> &ash::Entry {
        &*self.inner.platform_device_context.entry
    }

    pub(crate) fn vk_instance(&self) -> &ash::Instance {
        &self.inner.platform_device_context.instance
    }

    pub(crate) fn vk_device(&self) -> &ash::Device {
        &self.inner.platform_device_context.vk_device
    }

    pub(crate) fn vk_physical_device(&self) -> vk::PhysicalDevice {
        self.inner.platform_device_context.vk_physical_device
    }

    pub(crate) fn physical_device_info(&self) -> &PhysicalDeviceInfo {
        &self.inner.platform_device_context.physical_device_info
    }

    pub(crate) fn limits(&self) -> &vk::PhysicalDeviceLimits {
        &self
            .inner
            .platform_device_context
            .physical_device_info
            .properties
            .limits
    }

    pub(crate) fn vk_allocator(&self) -> &vk_mem::Allocator {
        &self.inner.platform_device_context.vk_allocator
    }

    pub(crate) fn queue_allocator(&self) -> &VkQueueAllocatorSet {
        &self.inner.platform_device_context.queue_allocator
    }

    pub(crate) fn vk_queue_family_indices(&self) -> &VkQueueFamilyIndices {
        &self
            .inner
            .platform_device_context
            .physical_device_info
            .queue_family_indices
    }

    pub(crate) fn dedicated_present_queue_lock(&self) -> &Mutex<()> {
        &self
            .inner
            .platform_device_context
            .dedicated_present_queue_lock
    }

    pub(crate) fn create_renderpass(
        device_context: &Self,
        renderpass_def: &VulkanRenderpassDef,
    ) -> GfxResult<VulkanRenderpass> {
        VulkanRenderpass::new(device_context, renderpass_def)
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
            _features: features,
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
        .fill_mode_non_solid(true)
        .sparse_binding(true)
        .sparse_residency_buffer(true)
        .fragment_stores_and_atomics(true)
        .wide_lines(true);

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
