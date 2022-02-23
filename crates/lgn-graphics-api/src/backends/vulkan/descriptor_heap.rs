use crate::{
    DescriptorHeapDef, DescriptorHeapPartition, DescriptorRef, DescriptorSet, DescriptorSetHandle,
    DescriptorSetLayout, DescriptorSetWriter, DeviceContext, GfxResult,
};

struct DescriptorHeapPoolConfig {
    pool_flags: ash::vk::DescriptorPoolCreateFlags,
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
            pool_flags: ash::vk::DescriptorPoolCreateFlags::empty(),
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
    fn create_pool(&self, device: &ash::Device) -> GfxResult<ash::vk::DescriptorPool> {
        let mut pool_sizes = Vec::with_capacity(16);

        fn add_if_not_zero(
            pool_sizes: &mut Vec<ash::vk::DescriptorPoolSize>,
            ty: ash::vk::DescriptorType,
            descriptor_count: u32,
        ) {
            if descriptor_count != 0 {
                pool_sizes.push(ash::vk::DescriptorPoolSize {
                    ty,
                    descriptor_count,
                });
            }
        }

        #[rustfmt::skip]
        {
            add_if_not_zero(&mut pool_sizes, ash::vk::DescriptorType::SAMPLER, self.samplers);
            add_if_not_zero(&mut pool_sizes, ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER, self.combined_image_samplers);
            add_if_not_zero(&mut pool_sizes, ash::vk::DescriptorType::SAMPLED_IMAGE, self.sampled_images);
            add_if_not_zero(&mut pool_sizes, ash::vk::DescriptorType::STORAGE_IMAGE, self.storage_images);
            add_if_not_zero(&mut pool_sizes, ash::vk::DescriptorType::UNIFORM_TEXEL_BUFFER, self.uniform_texel_buffers);
            add_if_not_zero(&mut pool_sizes, ash::vk::DescriptorType::STORAGE_TEXEL_BUFFER, self.storage_texel_buffers);
            add_if_not_zero(&mut pool_sizes, ash::vk::DescriptorType::UNIFORM_BUFFER, self.uniform_buffers);
            add_if_not_zero(&mut pool_sizes, ash::vk::DescriptorType::STORAGE_BUFFER, self.storage_buffers);
            add_if_not_zero(&mut pool_sizes, ash::vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC, self.dynamic_uniform_buffers);
            add_if_not_zero(&mut pool_sizes, ash::vk::DescriptorType::STORAGE_BUFFER_DYNAMIC, self.dynamic_storage_buffers);
            add_if_not_zero(&mut pool_sizes, ash::vk::DescriptorType::INPUT_ATTACHMENT, self.input_attachments);
        };

        let vk_pool = unsafe {
            device.create_descriptor_pool(
                &*ash::vk::DescriptorPoolCreateInfo::builder()
                    .flags(self.pool_flags)
                    .max_sets(self.descriptor_sets)
                    .pool_sizes(&pool_sizes),
                None,
            )?
        };

        Ok(vk_pool)
    }
}
/*
//
// VukanDescriptor
//

#[derive(Default)]
pub struct VulkanBufferDescriptor {
    vk_buffer_info: ash::vk::DescriptorBufferInfo,
}

impl BufferDescriptor {

    pub fn set_constant_buffer_platform(&mut self, buffer_view: &BufferView) {
        assert_eq!(
            std::alloc::Layout::new::<Self>(),
            std::alloc::Layout::new::<ash::vk::DescriptorBufferInfo>()
        );
        self.inner.vk_buffer_info.buffer = buffer_view.buffer().vk_buffer();
        self.inner.vk_buffer_info.offset = buffer_view.offset();
        self.inner.vk_buffer_info.range = buffer_view.size();
    }
}

#[derive(Default)]
pub struct VulkanTextureDescriptor {
    vk_image_info: ash::vk::DescriptorImageInfo,
}

#[derive(Default)]
pub struct VulkanSamplerDescriptor {
    vk_image_info: ash::vk::DescriptorImageInfo,
}

#[derive(Default)]
pub(crate) struct VulkanDescriptor {
    vk_write: ash::vk::WriteDescriptorSet,
}

impl VulkanDescriptor {
    pub(crate) fn new(
        descriptor_set_handle: DescriptorSetHandle,
        descriptor_def: &Descriptor,
        descriptor_array: &DescriptorArray,
    ) -> Self {
        let vk_descriptor_type = super::internal::shader_resource_type_to_descriptor_type(
            descriptor_def.shader_resource_type,
        );

        let mut write_descriptor_builder = ash::vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_set_handle.vk_type)
            .dst_binding(descriptor_def.binding)
            .dst_array_element(0)
            .descriptor_type(vk_descriptor_type);

        match descriptor_def.shader_resource_type {
            crate::ShaderResourceType::Sampler => match descriptor_array {
                DescriptorArray::Sampler(arr) => {
                    write_descriptor_builder.image_info(arr.as_slice())
                }
                DescriptorArray::Undefined(_)
                | DescriptorArray::Texture(_)
                | DescriptorArray::Buffer(_) => unreachable!(),
            },
            crate::ShaderResourceType::ConstantBuffer
            | crate::ShaderResourceType::StructuredBuffer
            | crate::ShaderResourceType::RWStructuredBuffer
            | crate::ShaderResourceType::ByteAdressBuffer
            | crate::ShaderResourceType::RWByteAdressBuffer => match descriptor_array {
                DescriptorArray::Buffer(arr) => {
                    write_descriptor_builder.buffer_info(arr.as_slice())
                }
                DescriptorArray::Undefined(_)
                | DescriptorArray::Texture(_)
                | DescriptorArray::Sampler(_) => unreachable!(),
            },
            crate::ShaderResourceType::Texture2D
            | crate::ShaderResourceType::RWTexture2D
            | crate::ShaderResourceType::Texture2DArray
            | crate::ShaderResourceType::RWTexture2DArray
            | crate::ShaderResourceType::Texture3D
            | crate::ShaderResourceType::RWTexture3D
            | crate::ShaderResourceType::TextureCube
            | crate::ShaderResourceType::TextureCubeArray => match descriptor_array {
                DescriptorArray::Texture(arr) => {
                    write_descriptor_builder.image_info(arr.as_slice())
                }
                DescriptorArray::Undefined(_)
                | DescriptorArray::Sampler(_)
                | DescriptorArray::Buffer(_) => unreachable!(),
            },
        }

        Self {
            vk_write: write_descriptor_builder.build(),
        }
    }
}
*/
//
// VulkanDescriptorHeap
//
pub(crate) struct VulkanDescriptorHeap {
    vk_pool: ash::vk::DescriptorPool,
}

impl VulkanDescriptorHeap {
    pub(crate) fn new(
        device_context: &DeviceContext,
        definition: &DescriptorHeapDef,
    ) -> GfxResult<Self> {
        let device = device_context.vk_device();
        let heap_pool_config: DescriptorHeapPoolConfig = definition.into();
        let vk_pool = heap_pool_config.create_pool(device)?;

        Ok(Self { vk_pool })
    }

    pub(crate) fn destroy(&self, device_context: &DeviceContext) {
        let device = device_context.vk_device();
        unsafe {
            device.destroy_descriptor_pool(self.vk_pool, None);
        }
    }
}

//
// VulkanDescriptorHeapPartition
//

pub(crate) struct VulkanDescriptorHeapPartition {
    vk_pool: ash::vk::DescriptorPool,
}

impl VulkanDescriptorHeapPartition {
    pub(crate) fn new(
        device_context: &DeviceContext,
        transient: bool,
        definition: &DescriptorHeapDef,
    ) -> GfxResult<Self> {
        let device = device_context.vk_device();
        let mut heap_pool_config: DescriptorHeapPoolConfig = definition.into();
        if !transient {
            heap_pool_config.pool_flags = ash::vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET
                | ash::vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND;
        }

        let vk_pool = heap_pool_config.create_pool(device)?;

        Ok(Self { vk_pool })
    }

    pub(crate) fn destroy(&self, device_context: &DeviceContext) {
        let device = device_context.vk_device();
        unsafe {
            device.destroy_descriptor_pool(self.vk_pool, None);
        }
    }
}

impl DescriptorHeapPartition {
    pub(crate) fn backend_reset(&self) -> GfxResult<()> {
        let device = self.inner.heap.inner.device_context.vk_device();
        unsafe {
            device
                .reset_descriptor_pool(
                    self.inner.backend_descriptor_heap_partition.vk_pool,
                    ash::vk::DescriptorPoolResetFlags::default(),
                )
                .map_err(Into::into)
        }
    }

    // pub(crate) fn backend_get_writer(
    //     &self,
    //     descriptor_set_layout: &DescriptorSetLayout,
    // ) -> GfxResult<DescriptorSetWriter> {
    //     let device = self.inner.heap.inner.device_context.vk_device();
    //     let allocate_info = ash::vk::DescriptorSetAllocateInfo::builder()
    //         .set_layouts(&[descriptor_set_layout.vk_layout()])
    //         .descriptor_pool(self.inner.backend_descriptor_heap_partition.vk_pool)
    //         .build();

    //     let result = unsafe { device.allocate_descriptor_sets(&allocate_info)? };

    //     DescriptorSetWriter::new(
    //         DescriptorSetHandle {
    //             backend_descriptor_set_handle: result[0],
    //         },
    //         descriptor_set_layout,
    //     )
    // }

    pub(crate) fn backend_alloc(&self, layout: &DescriptorSetLayout) -> GfxResult<DescriptorSet> {
        let device_context = &self.inner.heap.inner.device_context;
        let device = device_context.vk_device();
        let allocate_info = ash::vk::DescriptorSetAllocateInfo::builder()
            .set_layouts(&[layout.vk_layout()])
            .descriptor_pool(self.inner.backend_descriptor_heap_partition.vk_pool)
            .build();
        let result = unsafe { device.allocate_descriptor_sets(&allocate_info)? };
        let handle = DescriptorSetHandle {
            backend_descriptor_set_handle: result[0],
        };

        // Ok(writer.flush())
        Ok(DescriptorSet {
            layout: layout.clone(),
            handle,
        })
    }

    pub(crate) fn backend_write(
        &self,
        layout: &DescriptorSetLayout,
        descriptor_refs: &[DescriptorRef<'_>],
    ) -> GfxResult<DescriptorSetHandle> {
        let device_context = &self.inner.heap.inner.device_context;
        let device = device_context.vk_device();
        let allocate_info = ash::vk::DescriptorSetAllocateInfo::builder()
            .set_layouts(&[layout.vk_layout()])
            .descriptor_pool(self.inner.backend_descriptor_heap_partition.vk_pool)
            .build();
        let result = unsafe { device.allocate_descriptor_sets(&allocate_info)? };
        let descriptor_handle = DescriptorSetHandle {
            backend_descriptor_set_handle: result[0],
        };

        let mut writer = DescriptorSetWriter::new(descriptor_handle, layout);
        writer.set_descriptors(device_context, descriptor_refs);
        // Ok(writer.flush())
        Ok(descriptor_handle)
    }
}
