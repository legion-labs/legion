use crate::{DescriptorRef, DescriptorSetHandle, DescriptorSetLayout, DeviceContext, GfxError};

pub struct DescriptorSetWriter<'a> {
    pub(crate) device_context: &'a DeviceContext,
    pub(crate) descriptor_set_layout: &'a DescriptorSetLayout,
    pub(crate) descriptor_set: &'a DescriptorSetHandle,
}

impl<'a> DescriptorSetWriter<'a> {
    pub fn new(
        device_context: &'a DeviceContext,
        descriptor_set: &'a DescriptorSetHandle,
        descriptor_set_layout: &'a DescriptorSetLayout,
    ) -> Self {
        Self {
            device_context,
            descriptor_set_layout,
            descriptor_set,
        }
    }

    pub fn set_descriptors_by_name(&mut self, name: &str, descriptor_refs: &[DescriptorRef<'_>]) {
        let descriptor_index = self
            .descriptor_set_layout
            .find_descriptor_index_by_name(name)
            .ok_or_else(|| GfxError::from("Invalid descriptor name"))
            .unwrap();
        self.set_descriptors_by_index(descriptor_index, descriptor_refs);
    }

    pub fn set_descriptors_by_index(
        &mut self,
        descriptor_index: u32,
        descriptor_refs: &[DescriptorRef<'_>],
    ) {
        assert!(descriptor_index < self.descriptor_set_layout.descriptor_count());
        self.set_descriptors_by_index_and_offset(descriptor_index, 0, descriptor_refs);
    }

    pub fn set_descriptors_by_index_and_offset(
        &mut self,
        descriptor_index: u32,
        offset: u32,
        descriptor_refs: &[DescriptorRef<'_>],
    ) {
        assert!(descriptor_index < self.descriptor_set_layout.descriptor_count());
        assert!(
            offset
                < self
                    .descriptor_set_layout
                    .descriptor(descriptor_index)
                    .element_count
                    .get()
        );
        self.backend_set_descriptors_by_index_and_offset(descriptor_index, offset, descriptor_refs);
    }

    pub fn set_descriptors(&mut self, descriptor_refs: &[DescriptorRef<'_>]) {
        let flat_descriptor_count = self.descriptor_set_layout.flat_descriptor_count();
        assert_eq!(flat_descriptor_count as usize, descriptor_refs.len());
        self.backend_set_descriptors(descriptor_refs);
    }
}
