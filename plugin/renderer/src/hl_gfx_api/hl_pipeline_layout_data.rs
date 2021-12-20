use lgn_graphics_api::{Pipeline, MAX_DESCRIPTOR_SET_LAYOUTS};
use lgn_graphics_cgen_runtime::CGenPipelineLayoutDef;

use super::DescriptorSetData;

pub struct PipelineLayoutData<'rc> {
    info: &'static CGenPipelineLayoutDef,
    pipeline_state: &'rc Pipeline,
    descriptor_sets: [Option<&'rc DescriptorSetData>; MAX_DESCRIPTOR_SET_LAYOUTS],
}

impl<'rc> PipelineLayoutData<'rc> {
    pub fn new(info: &'static CGenPipelineLayoutDef, pipeline_state: &'rc Pipeline) -> Self {
        Self {
            info,
            pipeline_state,
            descriptor_sets: [None; MAX_DESCRIPTOR_SET_LAYOUTS],
        }
    }

    pub fn set_descriptor_set(&mut self, descriptor_set_data: &'rc DescriptorSetData) {
        let frequency = descriptor_set_data.frequency();
        assert!(
            self.info.descriptor_set_layout_ids[frequency]
                == descriptor_set_data.descriptor_set_id()
        );
        self.descriptor_sets[frequency] = Some(descriptor_set_data);
    }

    pub fn set_push_constant<T>(&mut self, data: &T) {}
}
