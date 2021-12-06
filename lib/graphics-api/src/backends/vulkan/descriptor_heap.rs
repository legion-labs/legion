use ash::vk;

use crate::{
    DescriptorHeapDef, DescriptorSetBufWriter, DescriptorSetHandle, DescriptorSetLayout,
    DeviceContext, GfxResult,
};

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

impl From<&DescriptorHeapDef> for DescriptorHeapPoolConfig {
    fn from(definition: &DescriptorHeapDef) -> Self {
        Self {
            pool_flags: vk::DescriptorPoolCreateFlags::empty(),
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
}

impl DescriptorHeapPoolConfig {
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

        let vk_pool = unsafe {
            device.create_descriptor_pool(
                &*vk::DescriptorPoolCreateInfo::builder()
                    .flags(self.pool_flags)
                    .max_sets(self.descriptor_sets)
                    .pool_sizes(&pool_sizes),
                None,
            )?
        };

        Ok(vk_pool)
    }
}

pub(crate) struct VulkanDescriptorHeap {
    vk_pool: vk::DescriptorPool,
}

impl VulkanDescriptorHeap {
    pub fn new(device_context: &DeviceContext, definition: &DescriptorHeapDef) -> GfxResult<Self> {
        let device = device_context.vk_device();
        let heap_pool_config: DescriptorHeapPoolConfig = definition.into();
        let vk_pool = heap_pool_config.create_pool(device)?;

        Ok(Self { vk_pool })
    }

    pub fn destroy(&self, device_context: &DeviceContext) {
        let device = device_context.vk_device();
        unsafe {
            device.destroy_descriptor_pool(self.vk_pool, None);
        }
    }

    pub fn reset(&self, device_context: &DeviceContext) -> GfxResult<()> {
        let device = device_context.vk_device();
        unsafe {
            device
                .reset_descriptor_pool(self.vk_pool, vk::DescriptorPoolResetFlags::default())
                .map_err(Into::into)
        }
    }

    pub fn allocate_descriptor_set(
        &self,
        device_context: &DeviceContext,
        descriptor_set_layout: &DescriptorSetLayout,
    ) -> GfxResult<DescriptorSetBufWriter> {
        let device = device_context.vk_device();
        let allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .set_layouts(&[descriptor_set_layout.platform_layout().vk_layout()])
            .descriptor_pool(self.vk_pool)
            .build();

        let result = unsafe { device.allocate_descriptor_sets(&allocate_info)? };

        DescriptorSetBufWriter::new(
            DescriptorSetHandle { vk_type: result[0] },
            descriptor_set_layout,
        )
    }
}
