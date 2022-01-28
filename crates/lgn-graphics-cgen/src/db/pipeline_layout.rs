use lgn_graphics_api::MAX_DESCRIPTOR_SET_LAYOUTS;

use super::{CGenTypeHandle, DescriptorSetHandle, ModelHandle, ModelObject};

#[derive(Debug, Clone)]
pub struct PushConstant {
    pub type_handle: CGenTypeHandle,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PipelineLayout {
    pub name: String,
    pub descriptor_sets: [Option<DescriptorSetHandle>; MAX_DESCRIPTOR_SET_LAYOUTS],
    pub push_constant: Option<CGenTypeHandle>,
}

pub type PipelineLayoutHandle = ModelHandle<PipelineLayout>;

impl PipelineLayout {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            descriptor_sets: [None; MAX_DESCRIPTOR_SET_LAYOUTS],
            push_constant: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn descriptor_sets(&self) -> impl Iterator<Item = &DescriptorSetHandle> + '_ {
        self.descriptor_sets
            .iter()
            .take_while(|ds_opt| ds_opt.is_some())
            .map(|ds_opt| ds_opt.as_ref().unwrap())
    }

    // pub fn find_descriptor_set_by_frequency(
    //     &self,
    //     model: &Model,
    //     frequency: usize,
    // ) -> &Option<DescriptorSetHandle> {
    //     &self.descriptor_sets[frequency]

    //     // for (_, content) in &self.members {
    //     //     match content {
    //     //         PipelineLayoutContent::DescriptorSet(ds_handle) => {
    //     //             let ds = ds_handle.get(model);
    //     //             if ds.frequency as usize == frequency {
    //     //                 return Some(*ds_handle);
    //     //             }
    //     //         }
    //     //         PipelineLayoutContent::PushConstant(_) => (),
    //     //     }
    //     // }
    //     // None
    // }
}

impl ModelObject for PipelineLayout {
    fn typename() -> &'static str {
        "PipelineLayout"
    }
    fn name(&self) -> &str {
        self.name.as_str()
    }
}
