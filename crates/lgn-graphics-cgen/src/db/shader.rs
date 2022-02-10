use std::{collections::HashSet, ops::RangeBounds};

use anyhow::{anyhow, Result};
use lgn_graphics_api::ShaderStageFlags;

use super::{Model, ModelObject, PipelineLayout, PipelineLayoutHandle};

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct Shader {
    name: String,
    path: String,
    stages: ShaderStageFlags,
    pipeline_layout: PipelineLayoutHandle,
    defines: Vec<String>,
}

impl Shader {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            path: String::new(),
            stages: ShaderStageFlags::NONE,
            pipeline_layout: PipelineLayoutHandle::invalid(),
            defines: Vec::new(),
        }
    }
}

impl ModelObject for Shader {
    fn typename() -> &'static str {
        "Shader"
    }
    fn name(&self) -> &str {
        &self.name
    }
}

pub struct ShaderBuilder<'mdl> {
    mdl: &'mdl Model,
    product: Shader,
    define_set: HashSet<String>,
}

impl<'mdl> ShaderBuilder<'mdl> {
    pub fn new(mdl: &'mdl Model, name: &str) -> Self {
        ShaderBuilder {
            mdl,
            product: Shader::new(name),
            define_set: HashSet::new(),
        }
    }

    pub fn set_path(mut self, path: &str) -> Result<Self> {
        self.product.path = path.to_owned();
        Ok(self)
    }

    pub fn add_stage(mut self, stage: &str) -> Result<Self> {
        match stage {
            "VS" => self.product.stages |= ShaderStageFlags::VERTEX_FLAG,
            "PS" => self.product.stages |= ShaderStageFlags::FRAGMENT_FLAG,
            "CS" => self.product.stages |= ShaderStageFlags::COMPUTE_FLAG,
            _ => {
                return Err(anyhow!("Invalid shader stage '{stage}'"));
            }
        }

        let use_graphic_pipeline = (self.product.stages
            & (ShaderStageFlags::VERTEX_FLAG | ShaderStageFlags::FRAGMENT_FLAG))
            != ShaderStageFlags::NONE;

        let use_compute_pipeline =
            (self.product.stages & ShaderStageFlags::COMPUTE_FLAG) != ShaderStageFlags::NONE;

        if use_graphic_pipeline && use_compute_pipeline {
            return Err(anyhow!("Conflicting shader stage '{stage}'"));
        }

        Ok(self)
    }

    pub fn set_pipeline_layout(mut self, pipeline_layout: &str) -> Result<Self> {
        let pl_handle = self
            .mdl
            .get_object_handle::<PipelineLayout>(pipeline_layout)
            .ok_or_else(|| anyhow!("Unknown PipelineLayout '{pipeline_layout}'"))?;

        self.product.pipeline_layout = pl_handle;

        Ok(self)
    }

    pub fn add_define(mut self, define: &str) -> Result<Self> {
        if !self.define_set.contains(define) {
            self.define_set.insert(define.to_owned());
            self.product.defines.push(define.to_owned());
        }
        Ok(self)
    }

    pub fn build(mut self) -> Result<Shader> {
        Ok(self.product)
    }
}
