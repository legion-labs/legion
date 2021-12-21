use lgn_graphics_api::{Pipeline, MAX_DESCRIPTOR_SET_LAYOUTS};
use lgn_graphics_cgen_runtime::CGenPipelineLayoutDef;

use super::DescriptorSetData;

pub struct PipelineLayoutData<'rc> {
    info: &'rc CGenPipelineLayoutDef,
    pipeline_state: &'rc Pipeline,
    descriptor_sets: [Option<&'rc DescriptorSetData<'rc>>; MAX_DESCRIPTOR_SET_LAYOUTS],
}

impl<'rc> PipelineLayoutData<'rc> {
    pub fn new(info: &'rc CGenPipelineLayoutDef, pipeline_state: &'rc Pipeline) -> Self {
        Self {
            info,
            pipeline_state,
            descriptor_sets: [None; MAX_DESCRIPTOR_SET_LAYOUTS],
        }
    }

    pub fn set_descriptor_set(&mut self, descriptor_set_data: &'rc DescriptorSetData<'rc>) {
        let frequency = descriptor_set_data.frequency();

        match self.info.descriptor_set_layout_ids[frequency as usize] {
            Some(e) => assert_eq!(e, descriptor_set_data.id()),
            None => panic!(),
        };

        self.descriptor_sets[frequency as usize] = Some(descriptor_set_data);
    }

    pub fn set_push_constant<T>(&mut self, data: &T) {}
}
