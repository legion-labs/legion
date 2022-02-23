#![allow(unsafe_code)]

use crate::{
    backends::{BackendDescriptorHeap, BackendDescriptorHeapPartition, BackendDescriptorSetHandle},
    deferred_drop::Drc,
    DescriptorRef, DescriptorSetLayout, DescriptorSetLayoutDef, DeviceContext, GfxResult,
    ShaderResourceType,
};

pub struct DescriptorSet {
    pub(crate) layout: DescriptorSetLayout,
    pub(crate) handle: DescriptorSetHandle,
}

/// Used to create a `DescriptorHeap`
#[derive(Default, Clone, Copy)]
pub struct DescriptorHeapDef {
    pub max_descriptor_sets: u32,
    pub sampler_count: u32,
    pub constant_buffer_count: u32,
    pub buffer_count: u32,
    pub rw_buffer_count: u32,
    pub texture_count: u32,
    pub rw_texture_count: u32,
}

impl DescriptorHeapDef {
    pub fn from_descriptor_set_layout_def(
        definition: &DescriptorSetLayoutDef,
        max_descriptor_sets: u32,
    ) -> Self {
        let mut result = Self {
            max_descriptor_sets,
            ..Self::default()
        };

        for descriptor_def in &definition.descriptor_defs {
            let count = max_descriptor_sets * descriptor_def.array_size_normalized();
            match descriptor_def.shader_resource_type {
                ShaderResourceType::Sampler => result.sampler_count += count,
                ShaderResourceType::ConstantBuffer => result.constant_buffer_count += count,
                ShaderResourceType::StructuredBuffer | ShaderResourceType::ByteAddressBuffer => {
                    result.buffer_count += count;
                }
                ShaderResourceType::RWStructuredBuffer
                | ShaderResourceType::RWByteAddressBuffer => {
                    result.rw_buffer_count += count;
                }
                ShaderResourceType::Texture2D
                | ShaderResourceType::Texture2DArray
                | ShaderResourceType::Texture3D
                | ShaderResourceType::TextureCube => result.texture_count += count,
                ShaderResourceType::RWTexture2D
                | ShaderResourceType::RWTexture2DArray
                | ShaderResourceType::RWTexture3D
                | ShaderResourceType::TextureCubeArray => result.rw_texture_count += count,
            }
        }

        result
    }
}

//
// DescriptorSetHandle
//
#[derive(Clone, Copy)]
pub struct DescriptorSetHandle {
    pub(crate) backend_descriptor_set_handle: BackendDescriptorSetHandle,
}

//
// DescriptorSetDataProvider
//

pub trait DescriptorSetDataProvider {
    fn layout(&self) -> &'static DescriptorSetLayout;
    fn frequency(&self) -> u32;
    fn descriptor_refs(&self, descriptor_index: usize) -> &[DescriptorRef<'_>];
}

//
// DescriptorHeapInner
//

pub(crate) struct DescriptorHeapInner {
    pub(crate) device_context: DeviceContext,

    pub(crate) backend_descriptor_heap: BackendDescriptorHeap,
}

impl Drop for DescriptorHeapInner {
    fn drop(&mut self) {
        self.backend_descriptor_heap.destroy(&self.device_context);
    }
}

//
// DescriptorHeap
//

#[derive(Clone)]
pub struct DescriptorHeap {
    pub(crate) inner: Drc<DescriptorHeapInner>,
}

impl DescriptorHeap {
    pub(crate) fn new(
        device_context: &DeviceContext,
        definition: &DescriptorHeapDef,
    ) -> GfxResult<Self> {
        let backend_descriptor_heap = BackendDescriptorHeap::new(device_context, definition)?;

        Ok(Self {
            inner: device_context
                .deferred_dropper()
                .new_drc(DescriptorHeapInner {
                    device_context: device_context.clone(),
                    backend_descriptor_heap,
                }),
        })
    }

    pub fn alloc_partition(
        &self,
        transient: bool,
        definition: &DescriptorHeapDef,
    ) -> GfxResult<DescriptorHeapPartition> {
        // todo(vdbdd): is there enough room inside this heap to allocate this partition
        DescriptorHeapPartition::new(self.clone(), transient, definition)
    }

    #[allow(clippy::unused_self)]
    #[allow(clippy::needless_pass_by_value)]
    #[allow(clippy::todo)]
    pub fn free_partition(&self, _partition: DescriptorHeapPartition) {
        // todo(vdbdd): free
    }
}

//
// DescriptorHeapPartitionInner
//

pub(crate) struct DescriptorHeapPartitionInner {
    pub(crate) heap: DescriptorHeap,
    pub(crate) transient: bool,
    pub(crate) backend_descriptor_heap_partition: BackendDescriptorHeapPartition,
}

impl Drop for DescriptorHeapPartitionInner {
    fn drop(&mut self) {
        self.backend_descriptor_heap_partition
            .destroy(&self.heap.inner.device_context);
    }
}

//
// DescriptorHeapPartition
//

pub struct DescriptorHeapPartition {
    pub(crate) inner: Box<DescriptorHeapPartitionInner>,
}

impl DescriptorHeapPartition {
    pub(crate) fn new(
        heap: DescriptorHeap,
        transient: bool,
        definition: &DescriptorHeapDef,
    ) -> GfxResult<Self> {
        let platform_descriptor_heap_partition =
            BackendDescriptorHeapPartition::new(&heap.inner.device_context, transient, definition)?;
        Ok(Self {
            inner: Box::new(DescriptorHeapPartitionInner {
                heap,
                transient,
                backend_descriptor_heap_partition: platform_descriptor_heap_partition,
            }),
        })
    }

    pub fn reset(&self) -> GfxResult<()> {
        assert!(self.inner.transient);
        self.backend_reset()
    }

    pub fn transient(&self) -> bool {
        self.inner.transient
    }

    pub fn alloc(&self, layout: &DescriptorSetLayout) -> GfxResult<DescriptorSet> {
        assert!(!self.inner.transient);
        self.backend_alloc(layout)
    }

    // pub fn get_writer<'frame>(
    //     &self,
    //     descriptor_set_layout: &DescriptorSetLayout,
    //     bump: &'frame bumpalo::Bump,
    // ) -> GfxResult<DescriptorSetWriter<'frame>> {
    //     self.backend_get_writer(descriptor_set_layout, bump)
    // }

    pub fn write(
        &self,
        layout: &DescriptorSetLayout,
        descriptor_refs: &[DescriptorRef<'_>],
    ) -> GfxResult<DescriptorSetHandle> {
        assert!(self.inner.transient);
        self.backend_write(layout, descriptor_refs)
    }
}
