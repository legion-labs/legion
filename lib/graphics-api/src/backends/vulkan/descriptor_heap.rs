use std::sync::{Arc, Mutex};

use crate::{DescriptorHeap, DescriptorHeapDef, GfxResult, VulkanApi};
use ash::vk;

use super::VulkanDeviceContext;

struct DescriptorHeapPoolConfig {
    pool_flags: vk::DescriptorPoolCreateFlags,
    descriptor_sets: u32,
    samplers: u32,
    combined_image_samplers: u32,
    sampled_images: u32,
    storage_images: u32,
    uniform_texel_buffers: u32,
    storage_texel_buffers: u32,
    uniform_buffers: u32,
    storage_buffers: u32,
    dynamic_uniform_buffers: u32,
    dynamic_storage_buffers: u32,
    input_attachments: u32,
}

impl DescriptorHeapPoolConfig {
    fn new(definition: &DescriptorHeapDef) -> Self {
        DescriptorHeapPoolConfig {
            pool_flags: if definition.transient {
                vk::DescriptorPoolCreateFlags::empty()
            } else {
                vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET
            },
            descriptor_sets: definition.max_descriptor_sets,
            samplers: definition.sampler_count,
            combined_image_samplers: 0,
            sampled_images: definition.texture_count,
            storage_images: definition.rw_texture_count,
            uniform_texel_buffers: 0,
            storage_texel_buffers: 0,
            uniform_buffers: definition.constant_buffer_count,
            storage_buffers: definition.buffer_count + definition.rw_buffer_count,
            dynamic_uniform_buffers: 0,
            dynamic_storage_buffers: 0,
            input_attachments: 0,
        }
    }

    fn create_pool(&self, device: &ash::Device) -> GfxResult<vk::DescriptorPool> {
        let mut pool_sizes = Vec::with_capacity(16);

        fn add_if_not_zero(
            pool_sizes: &mut Vec<vk::DescriptorPoolSize>,
            ty: vk::DescriptorType,
            descriptor_count: u32,
        ) {
            if descriptor_count != 0 {
                pool_sizes.push(vk::DescriptorPoolSize {
                    ty,
                    descriptor_count,
                });
            }
        }

        #[rustfmt::skip]
        {
            add_if_not_zero(&mut pool_sizes, vk::DescriptorType::SAMPLER, self.samplers);
            add_if_not_zero(&mut pool_sizes, vk::DescriptorType::COMBINED_IMAGE_SAMPLER, self.combined_image_samplers);
            add_if_not_zero(&mut pool_sizes, vk::DescriptorType::SAMPLED_IMAGE, self.sampled_images);
            add_if_not_zero(&mut pool_sizes, vk::DescriptorType::STORAGE_IMAGE, self.storage_images);
            add_if_not_zero(&mut pool_sizes, vk::DescriptorType::UNIFORM_TEXEL_BUFFER, self.uniform_texel_buffers);
            add_if_not_zero(&mut pool_sizes, vk::DescriptorType::STORAGE_TEXEL_BUFFER, self.storage_texel_buffers);
            add_if_not_zero(&mut pool_sizes, vk::DescriptorType::UNIFORM_BUFFER, self.uniform_buffers);
            add_if_not_zero(&mut pool_sizes, vk::DescriptorType::STORAGE_BUFFER, self.storage_buffers);
            add_if_not_zero(&mut pool_sizes, vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC, self.dynamic_uniform_buffers);
            add_if_not_zero(&mut pool_sizes, vk::DescriptorType::STORAGE_BUFFER_DYNAMIC, self.dynamic_storage_buffers);
            add_if_not_zero(&mut pool_sizes, vk::DescriptorType::INPUT_ATTACHMENT, self.input_attachments);
        };

        unsafe {
            Ok(device.create_descriptor_pool(
                &*vk::DescriptorPoolCreateInfo::builder()
                    .flags(self.pool_flags)
                    .max_sets(self.descriptor_sets)
                    .pool_sizes(&pool_sizes),
                None,
            )?)
        }
    }
}

struct DescriptorHeapVulkanInner {
    device_context: VulkanDeviceContext,
    pub definition: DescriptorHeapDef,
    pools: Vec<vk::DescriptorPool>,
}

impl Drop for DescriptorHeapVulkanInner {
    fn drop(&mut self) {
        // Assert that everything was destroyed. (We can't do it automatically since we don't have
        // a reference to the device)
        // assert!(self.pools.is_empty());
        let device = self.device_context.device();
        for pool in &self.pools {
            unsafe {
                device.destroy_descriptor_pool(*pool, None);
            }
        }
    }
}

// This is an endlessly growing descriptor pools. New pools are allocated in large chunks as needed.
// It also takes locks on every operation. So it's better to allocate large chunks of descriptors
// and pool/reuse them.
#[derive(Clone)]
pub struct VulkanDescriptorHeap {
    inner: Arc<Mutex<DescriptorHeapVulkanInner>>,
}

impl DescriptorHeap<VulkanApi> for VulkanDescriptorHeap {}

impl VulkanDescriptorHeap {
    pub(crate) fn new(
        device_context: &VulkanDeviceContext,
        definition: &DescriptorHeapDef,
    ) -> GfxResult<Self> {
        let device = device_context.device();
        let heap_pool_config = DescriptorHeapPoolConfig::new(definition);
        let pool = heap_pool_config.create_pool(device)?;

        let inner = DescriptorHeapVulkanInner {
            device_context: device_context.clone(),
            definition: *definition,
            pools: vec![pool],
        };

        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
        })
    }

    pub(crate) fn allocate_descriptor_sets(
        &self,
        device: &ash::Device,
        set_layouts: &[vk::DescriptorSetLayout],
    ) -> GfxResult<Vec<vk::DescriptorSet>> {
        let mut heap = self.inner.lock().unwrap();

        let mut allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .set_layouts(set_layouts)
            .build();

        // Heap might have been cleared
        if !heap.pools.is_empty() {
            let pool = *heap.pools.last().unwrap();
            allocate_info.descriptor_pool = pool;

            let result = unsafe { device.allocate_descriptor_sets(&allocate_info) };

            // If successful bail, otherwise allocate a new pool below
            if let Ok(result) = result {
                return Ok(result);
            }
        }

        // We either didn't have any pools, or assume the pool wasn't large enough. Create a new
        // pool and try again
        let heap_pool_config = DescriptorHeapPoolConfig::new(&heap.definition);
        let new_pool = heap_pool_config.create_pool(device)?;
        heap.pools.push(new_pool);

        let pool = *heap.pools.last().unwrap();
        allocate_info.descriptor_pool = pool;
        Ok(unsafe { device.allocate_descriptor_sets(&allocate_info)? })
    }
}
