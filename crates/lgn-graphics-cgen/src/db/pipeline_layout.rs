use super::{CGenTypeHandle, DescriptorSetHandle, Model, ModelHandle, ModelObject};

#[derive(Debug, Clone)]
pub struct PushConstant {
    pub name: String,
    pub type_handle: CGenTypeHandle,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum PipelineLayoutContent {
    DescriptorSet(DescriptorSetHandle),
    PushConstant(CGenTypeHandle),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PipelineLayout {
    pub name: String,
    pub members: Vec<(String, PipelineLayoutContent)>,
}

pub type PipelineLayoutHandle = ModelHandle<PipelineLayout>;

impl PipelineLayout {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            members: Vec::new(),
        }
    }

    pub fn descriptor_sets(&self) -> impl Iterator<Item = DescriptorSetHandle> + '_ {
        let x = self.members.iter().filter_map(|m| match m.1 {
            PipelineLayoutContent::DescriptorSet(ds) => Some(ds),
            PipelineLayoutContent::PushConstant(_) => None,
        });
        x
    }

    pub fn push_constant(&self) -> Option<CGenTypeHandle> {
        let x = self
            .members
            .iter()
            .filter_map(|m| match m.1 {
                PipelineLayoutContent::PushConstant(ds) => Some(ds),
                PipelineLayoutContent::DescriptorSet(_) => None,
            })
            .last();
        x
    }

    pub fn find_descriptor_set_by_frequency(
        &self,
        model: &Model,
        frequency: usize,
    ) -> Option<DescriptorSetHandle> {
        for (_, content) in &self.members {
            match content {
                PipelineLayoutContent::DescriptorSet(ds_handle) => {
                    let ds = ds_handle.get(model);
                    if ds.frequency as usize == frequency {
                        return Some(*ds_handle);
                    }
                }
                PipelineLayoutContent::PushConstant(_) => (),
            }
        }
        None
    }
}

impl ModelObject for PipelineLayout {
    fn typename() -> &'static str {
        "PipelineLayout"
    }
    fn name(&self) -> &str {
        self.name.as_str()
    }
}
