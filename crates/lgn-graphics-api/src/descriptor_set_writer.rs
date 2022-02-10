use lgn_tracing::error;

use crate::{
    backends::BackendDescriptorSetWriter, DescriptorRef, DescriptorSetDataProvider,
    DescriptorSetHandle, DescriptorSetLayout, DeviceContext, GfxError, GfxResult,
    MAX_DESCRIPTOR_BINDINGS,
};

pub struct DescriptorSetWriter<'frame> {
    pub(crate) descriptor_set: DescriptorSetHandle,
    pub(crate) descriptor_set_layout: DescriptorSetLayout,
    pub(crate) backend_write: BackendDescriptorSetWriter<'frame>,
    write_mask: u64, // max number of bindings: 64
}

impl<'frame> DescriptorSetWriter<'frame> {
    pub fn new(
        descriptor_set: DescriptorSetHandle,
        descriptor_set_layout: &DescriptorSetLayout,
        bump: &'frame bumpalo::Bump,
    ) -> GfxResult<Self> {
        let backend_write = BackendDescriptorSetWriter::new(descriptor_set_layout, bump)?;

        Ok(Self {
            descriptor_set,
            descriptor_set_layout: descriptor_set_layout.clone(),
            backend_write,
            write_mask: descriptor_set_layout.binding_mask(),
        })
    }

    pub fn set_descriptors_by_name(
        &mut self,
        name: &str,
        update_datas: &[DescriptorRef<'_>],
    ) -> GfxResult<()> {
        let descriptor_index = self
            .descriptor_set_layout
            .find_descriptor_index_by_name(name)
            .ok_or_else(|| GfxError::from("Invalid descriptor name"))?;

        self.set_descriptors_by_index(descriptor_index, update_datas);
        Ok(())
    }

    pub fn set_descriptors_by_index(&mut self, index: usize, update_datas: &[DescriptorRef<'_>]) {
        let descriptor = self.descriptor_set_layout.descriptor(index);
        self.write_mask &= !(1u64 << descriptor.binding);
        self.backend_set_descriptors_by_index(index, update_datas);
    }

    pub fn set_descriptors(&mut self, descriptor_set: &impl DescriptorSetDataProvider) {
        let descriptor_count = self
            .descriptor_set_layout
            .definition()
            .descriptor_defs
            .len();

        for index in 0..descriptor_count {
            let descriptor_refs = descriptor_set.descriptor_refs(index);
            self.set_descriptors_by_index(index, descriptor_refs);
        }
    }

    pub fn flush(self, device_context: &DeviceContext) -> DescriptorSetHandle {
        if self.write_mask != 0 {
            error!(
                "An instance of DescriptorSetWriter cannot be flushed due to missing descriptors"
            );
            for i in 0..MAX_DESCRIPTOR_BINDINGS {
                let mask = 1u64 << i;
                if (self.write_mask & mask) != 0 {
                    error!("{:?}", self.descriptor_set_layout.descriptor(i));
                }
            }
            panic!();
        }

        self.backend_flush(device_context);
        self.descriptor_set
    }
}
