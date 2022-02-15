use lgn_graphics_api::MAX_DESCRIPTOR_SET_LAYOUTS;

use super::{
    CGenType, CGenTypeHandle, DescriptorSet, DescriptorSetHandle, Model, ModelHandle, ModelObject,
};

use anyhow::{anyhow, Context, Result};

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
}

impl ModelObject for PipelineLayout {
    fn typename() -> &'static str {
        "PipelineLayout"
    }
    fn name(&self) -> &str {
        self.name.as_str()
    }
}

pub struct PipelineLayoutBuilder<'mdl> {
    mdl: &'mdl Model,
    product: PipelineLayout,
}

impl<'mdl> PipelineLayoutBuilder<'mdl> {
    pub fn new(mdl: &'mdl Model, name: &str) -> Self {
        PipelineLayoutBuilder {
            mdl,
            product: PipelineLayout::new(name),
        }
    }

    /// Add `DescriptorSet`.
    ///
    /// # Errors
    /// todo
    pub fn add_descriptor_set(mut self, ds_ty: &str) -> Result<Self> {
        // check descriptor_set exists
        let ds_handle = self.mdl.get_object_handle::<DescriptorSet>(ds_ty);
        if ds_handle.is_none() {
            return Err(anyhow!("Unknown DescriptorSet '{}'", ds_ty,));
        }
        let ds_handle = ds_handle.unwrap();
        let ds = ds_handle.get(self.mdl);

        // check for frequency conflict
        if self.product.descriptor_sets[ds.frequency as usize].is_some() {
            return Err(anyhow!("Frequency conflict for DescriptorSet '{}'", ds_ty,));
        }
        self.product.descriptor_sets[ds.frequency as usize] = Some(ds_handle);

        Ok(self)
    }

    /// Add `PushConstant`.
    ///
    /// # Errors
    /// todo
    pub fn add_push_constant(mut self, typename: &str) -> Result<Self> {
        // only one push_constant is allowed
        if self.product.push_constant.is_some() {
            return Err(anyhow!("Only one PushConstant allowed",));
        }
        // get cgen type and check its existence if necessary
        let ty_handle = self
            .mdl
            .get_object_handle::<CGenType>(typename)
            .context(anyhow!("Unknown type '{}' for PushConstant", typename,))?;
        let cgen_type = ty_handle.get(self.mdl);
        // Only struct types allowed for now
        if let CGenType::Struct(_def) = cgen_type {
        } else {
            return Err(anyhow!("PushConstant must be Struct types "));
        }

        // done
        self.product.push_constant = Some(ty_handle);

        Ok(self)
    }

    /// build
    ///
    /// # Errors
    /// todo
    #[allow(clippy::unnecessary_wraps)]
    pub fn build(self) -> Result<PipelineLayout> {
        let mut first_none = None;

        for i in 0..self.product.descriptor_sets.len() {
            if self.product.descriptor_sets[i].is_none() && first_none.is_none() {
                first_none = Some(i);
            } else if self.product.descriptor_sets[i].is_some() && first_none.is_some() {
                return Err(anyhow!("DescriptorSets must be contiguous",));
            }
        }

        Ok(self.product)
    }
}
