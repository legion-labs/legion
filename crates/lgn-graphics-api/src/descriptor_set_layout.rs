use std::{num::NonZeroU32, sync::atomic::Ordering};

use crate::{
    backends::BackendDescriptorSetLayout, deferred_drop::Drc, DeviceContext, GfxResult,
    ShaderResourceType, MAX_DESCRIPTOR_BINDINGS,
};

static NEXT_DESCRIPTOR_SET_LAYOUT_ID: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(1);

#[derive(Clone, Debug)]
pub struct Descriptor {
    pub name: String,
    pub shader_resource_type: ShaderResourceType,
    pub bindless: bool,
    pub flat_index: u32,
    pub element_count: NonZeroU32,
}

#[derive(Debug, Clone, Hash)]
pub struct DescriptorDef {
    pub name: String,
    pub bindless: bool,
    pub shader_resource_type: ShaderResourceType,
    pub array_size: u32,
}

impl DescriptorDef {
    pub fn array_size_normalized(&self) -> u32 {
        self.array_size.max(1u32)
    }
}

#[derive(Debug, Clone, Hash)]
pub struct DescriptorSetLayoutDef {
    pub frequency: u32,
    pub descriptor_defs: Vec<DescriptorDef>,
}

impl DescriptorSetLayoutDef {
    pub fn new() -> Self {
        Self {
            frequency: 0,
            descriptor_defs: Vec::new(),
        }
    }
}

impl Default for DescriptorSetLayoutDef {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug)]
pub(crate) struct DescriptorSetLayoutInner {
    device_context: DeviceContext,
    definition: DescriptorSetLayoutDef,
    id: u32,
    frequency: u32,
    descriptors: Vec<Descriptor>,
    flat_descriptor_count: u32,

    pub(crate) backend_layout: BackendDescriptorSetLayout,
}

impl Drop for DescriptorSetLayoutInner {
    fn drop(&mut self) {
        self.backend_layout.destroy(&self.device_context);
    }
}

#[derive(Debug, Clone)]
pub struct DescriptorSetLayout {
    pub(crate) inner: Drc<DescriptorSetLayoutInner>,
}

impl DescriptorSetLayout {
    pub fn new(
        device_context: &DeviceContext,
        definition: &DescriptorSetLayoutDef,
    ) -> GfxResult<Self> {
        assert!(definition.descriptor_defs.len() < MAX_DESCRIPTOR_BINDINGS);

        let mut flat_descriptor_count = 0;
        let mut descriptors = Vec::new();

        for descriptor_def in &definition.descriptor_defs {
            let element_count = descriptor_def.array_size_normalized();

            let descriptor = Descriptor {
                name: descriptor_def.name.clone(),
                bindless: descriptor_def.bindless,
                shader_resource_type: descriptor_def.shader_resource_type,
                element_count: NonZeroU32::new(element_count).unwrap(),
                flat_index: flat_descriptor_count,
            };

            flat_descriptor_count += element_count;

            descriptors.push(descriptor);
        }

        let backend_layout = BackendDescriptorSetLayout::new(device_context, &descriptors)?;

        let descriptor_set_layout_id =
            NEXT_DESCRIPTOR_SET_LAYOUT_ID.fetch_add(1, Ordering::Relaxed);

        let result = Self {
            inner: device_context
                .deferred_dropper()
                .new_drc(DescriptorSetLayoutInner {
                    device_context: device_context.clone(),
                    definition: definition.clone(),
                    id: descriptor_set_layout_id,
                    frequency: definition.frequency,
                    descriptors,
                    flat_descriptor_count,
                    backend_layout,
                }),
        };

        Ok(result)
    }

    pub fn device_context(&self) -> &DeviceContext {
        &self.inner.device_context
    }

    pub fn definition(&self) -> &DescriptorSetLayoutDef {
        &self.inner.definition
    }

    pub fn uid(&self) -> u32 {
        self.inner.id
    }

    pub fn frequency(&self) -> u32 {
        self.inner.frequency
    }

    pub fn descriptor_count(&self) -> u32 {
        self.inner.descriptors.len() as u32
    }

    pub fn descriptor(&self, index: u32) -> &Descriptor {
        &self.inner.descriptors[index as usize]
    }

    pub fn find_descriptor_index_by_name(&self, name: &str) -> Option<u32> {
        self.inner
            .descriptors
            .iter()
            .position(|descriptor| name == descriptor.name)
            .map(|x| x as u32)
    }

    pub(crate) fn flat_descriptor_count(&self) -> u32 {
        self.inner.flat_descriptor_count
    }
}

impl PartialEq for DescriptorSetLayout {
    fn eq(&self, other: &Self) -> bool {
        self.inner.id == other.inner.id
    }
}
