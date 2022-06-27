#[cfg(target_os = "windows")]
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
#[cfg(debug_assertions)]
#[cfg(feature = "track-device-contexts")]
use std::sync::Mutex;

use lgn_tracing::trace;
use raw_window_handle::HasRawWindowHandle;

use super::deferred_drop::DeferredDropper;
use crate::backends::BackendDeviceContext;
use crate::{
    ApiDef, Buffer, BufferDef, ComputePipelineDef, DescriptorHeap, DescriptorHeapDef,
    DescriptorSetLayout, DescriptorSetLayoutDef, ExtensionMode, Fence, GfxResult,
    GraphicsPipelineDef, Instance, Pipeline, Queue, QueueType, RootSignature, RootSignatureDef,
    Sampler, SamplerDef, Semaphore, SemaphoreDef, Shader, ShaderModule, ShaderModuleDef,
    ShaderStageDef, Swapchain, SwapchainDef, Texture, TextureDef,
};

/// Used to specify which type of physical device is preferred. It's recommended
/// to read the Vulkan spec to understand precisely what these types mean
///
/// Values here match `VkPhysicalDeviceType`, `DiscreteGpu` is the recommended
/// default
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

/// Information about the device, mostly limits, requirements (like memory
/// alignment), and flags to indicate whether certain features are supported
#[derive(Clone, Copy)]
pub struct DeviceInfo {
    pub supports_multithreaded_usage: bool,

    pub min_uniform_buffer_offset_alignment: u32,
    pub min_storage_buffer_offset_alignment: u32,
    pub upload_buffer_texture_alignment: u32,
    pub upload_buffer_texture_row_alignment: u32,

    // Requires iOS 14.0, macOS 10.12
    pub supports_clamp_to_border_color: bool,

    pub max_vertex_attribute_count: u32,
    //max_vertex_input_binding_count: u32,
    // max_root_signature_dwords: u32,
    // wave_lane_count: u32,
    // wave_ops_support_flags: u32,
    // gpu_vendor_preset: u32,
    // metal_argument_buffer_max_textures: u32,
    // metal_heaps: u32,
    // metal_placement_heaps: u32,
    // metal_draw_index_vertex_offset_supported: bool,
}

pub(crate) struct DeviceContextInner {
    device_info: DeviceInfo,
    deferred_dropper: DeferredDropper,
    destroyed: AtomicBool,
    current_cpu_frame: AtomicU64,

    #[cfg(debug_assertions)]
    #[cfg(feature = "track-device-contexts")]
    next_create_index: AtomicU64,

    #[cfg(debug_assertions)]
    #[cfg(feature = "track-device-contexts")]
    pub(crate) all_contexts: Mutex<fnv::FnvHashMap<u64, backtrace::Backtrace>>,

    pub(crate) backend_device_context: BackendDeviceContext,
}

impl std::fmt::Debug for DeviceContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeviceContext")
            .field("handle", &0_u64)
            .finish()
    }
}

impl Drop for DeviceContextInner {
    fn drop(&mut self) {
        if !self.destroyed.swap(true, Ordering::AcqRel) {
            trace!("destroying device");
            self.deferred_dropper.destroy();

            self.backend_device_context.destroy();

            //self.surface_loader.destroy_surface(self.surface, None);
            trace!("destroyed device");
        }
    }
}

impl DeviceContextInner {
    pub fn new(instance: &Instance<'_>, windowing_mode: ExtensionMode) -> GfxResult<Self> {
        #[cfg(debug_assertions)]
        #[cfg(feature = "track-device-contexts")]
        let all_contexts = {
            let create_backtrace = backtrace::Backtrace::new_unresolved();
            let mut all_contexts = fnv::FnvHashMap::<u64, backtrace::Backtrace>::default();
            all_contexts.insert(0, create_backtrace);
            all_contexts
        };

        let (backend_device_context, device_info) =
            BackendDeviceContext::new(instance.backend_instance, windowing_mode)?;

        Ok(Self {
            device_info,
            deferred_dropper: DeferredDropper::new(3),
            destroyed: AtomicBool::new(false),
            current_cpu_frame: AtomicU64::new(0),

            backend_device_context,

            #[cfg(debug_assertions)]
            #[cfg(feature = "track-device-contexts")]
            all_contexts: Mutex::new(all_contexts),

            #[cfg(debug_assertions)]
            #[cfg(feature = "track-device-contexts")]
            next_create_index: AtomicU64::new(1),
        })
    }
}

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

            trace!("Cloned VulkanDeviceContext create_index {}", create_index);
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
    pub fn new(instance: &Instance<'_>, api_def: &ApiDef) -> GfxResult<Self> {
        let inner = Arc::new(DeviceContextInner::new(instance, api_def.windowing_mode)?);

        Ok(Self {
            inner,
            #[cfg(debug_assertions)]
            #[cfg(feature = "track-device-contexts")]
            create_index: 0,
        })
    }

    pub fn create_queue(&self, queue_type: QueueType) -> Queue {
        Queue::new(self, queue_type)
    }

    pub fn create_fence(&self) -> Fence {
        Fence::new(self)
    }

    pub fn create_semaphore(&self, semaphore_def: SemaphoreDef) -> Semaphore {
        Semaphore::new(self, semaphore_def)
    }

    pub fn create_swapchain(
        &self,
        raw_window_handle: &dyn HasRawWindowHandle,
        swapchain_def: SwapchainDef,
    ) -> Swapchain {
        Swapchain::new(self, raw_window_handle, swapchain_def)
    }

    pub fn create_sampler(&self, sampler_def: SamplerDef) -> Sampler {
        Sampler::new(self, sampler_def)
    }

    pub fn create_texture<T: AsRef<str>>(&self, texture_def: TextureDef, debug_name: T) -> Texture {
        let texture = Texture::new(self, texture_def);
        self.set_texture_name(&texture, debug_name);
        texture
    }

    pub fn create_buffer<T: AsRef<str>>(&self, buffer_def: BufferDef, debug_name: T) -> Buffer {
        let buffer = Buffer::new(self, buffer_def);
        self.set_buffer_name(&buffer, debug_name);
        buffer
    }

    pub fn create_shader(&self, stages: Vec<ShaderStageDef>) -> Shader {
        Shader::new(self, stages)
    }

    pub fn create_descriptorset_layout(
        &self,
        descriptorset_layout_def: DescriptorSetLayoutDef,
    ) -> DescriptorSetLayout {
        DescriptorSetLayout::new(self, descriptorset_layout_def)
    }

    pub fn create_root_signature(&self, root_signature_def: RootSignatureDef) -> RootSignature {
        RootSignature::new(self, root_signature_def)
    }

    pub fn create_descriptor_heap(&self, descriptor_heap_def: DescriptorHeapDef) -> DescriptorHeap {
        DescriptorHeap::new(self, descriptor_heap_def)
    }

    pub fn create_graphics_pipeline(&self, graphics_pipeline_def: GraphicsPipelineDef) -> Pipeline {
        Pipeline::new_graphics_pipeline(self, graphics_pipeline_def)
    }

    pub fn create_compute_pipeline(&self, compute_pipeline_def: ComputePipelineDef) -> Pipeline {
        Pipeline::new_compute_pipeline(self, compute_pipeline_def)
    }

    pub fn create_shader_module(&self, data: ShaderModuleDef<'_>) -> GfxResult<ShaderModule> {
        ShaderModule::new(self, data)
    }

    pub(crate) fn deferred_dropper(&self) -> &DeferredDropper {
        &self.inner.deferred_dropper
    }

    pub fn free_gpu_memory(&self) {
        self.inner.deferred_dropper.flush();
    }

    pub fn device_info(&self) -> &DeviceInfo {
        &self.inner.device_info
    }

    pub fn inc_current_cpu_frame(&self) {
        self.inner.current_cpu_frame.fetch_add(1, Ordering::Relaxed);
    }

    pub fn current_cpu_frame(&self) -> u64 {
        self.inner.current_cpu_frame.load(Ordering::Relaxed)
    }

    pub fn set_texture_name<T: AsRef<str>>(&self, texture: &Texture, name: T) {
        self.inner
            .backend_device_context
            .set_texture_name(texture, name.as_ref());
    }

    pub fn set_buffer_name<T: AsRef<str>>(&self, buffer: &Buffer, name: T) {
        self.inner
            .backend_device_context
            .set_buffer_name(buffer, name.as_ref());
    }
}

pub enum ExternalResourceType {
    Image,
    Semaphore,
}

#[cfg(target_os = "windows")]
pub type ExternalResourceHandle = *mut c_void;
#[cfg(target_os = "linux")]
pub type ExternalResourceHandle = i32;

pub trait ExternalResource<T> {
    fn clone_resource(&self) -> T;

    fn external_resource_type() -> ExternalResourceType;

    fn external_resource_handle(&self, device_context: &DeviceContext) -> ExternalResourceHandle;
}
