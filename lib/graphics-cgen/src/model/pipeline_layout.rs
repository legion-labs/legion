use super::{CGenTypeRef, DescriptorSetRef, Model, ModelHandle, ModelObject};

#[derive(Debug, Clone)]
pub struct PushConstant {
    pub name: String,
    pub ty_ref: CGenTypeRef,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum PipelineLayoutContent {
    DescriptorSet(DescriptorSetRef),
    Pushconstant(CGenTypeRef),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PipelineLayout {
    pub name: String,
    pub members: Vec<(String, PipelineLayoutContent)>,
}

pub type PipelineLayoutRef = ModelHandle<PipelineLayout>;

impl PipelineLayout {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            members: Vec::new(),
        }
    }

    pub fn descriptor_sets(&self) -> impl Iterator<Item = DescriptorSetRef> + '_ {
        let x = self.members.iter().filter_map(|m| match m.1 {
            PipelineLayoutContent::DescriptorSet(ds) => Some(ds),
            PipelineLayoutContent::Pushconstant(_) => None,
        });
        x
    }

    pub fn push_constant(&self) -> Option<CGenTypeRef> {
        let x = self
            .members
            .iter()
            .filter_map(|m| match m.1 {
                PipelineLayoutContent::Pushconstant(ds) => Some(ds),
                PipelineLayoutContent::DescriptorSet(_) => None,
            })
            .last();
        x
    }

    pub fn find_descriptor_set_by_frequency(
        &self,
        model: &Model,
        frequency: usize,
    ) -> Option<DescriptorSetRef> {
        for (_, content) in &self.members {
            match content {
                PipelineLayoutContent::DescriptorSet(ds_ref) => {
                    let ds = ds_ref.get(model);
                    if ds.frequency as usize == frequency {
                        return Some(*ds_ref);
                    }
                }
                PipelineLayoutContent::Pushconstant(_) => (),
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
