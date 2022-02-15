use std::collections::HashSet;

use anyhow::{anyhow, Result};
use lgn_graphics_api::ShaderStage;

use super::{Model, ModelObject, PipelineLayout, PipelineLayoutHandle};

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct Shader {
    pub name: String,
    pub path: String,
    pub pipeline_layout: PipelineLayoutHandle,
    pub options: Vec<String>,
    pub instances: Vec<ShaderInstance>,
}

impl Shader {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            path: String::new(),
            pipeline_layout: PipelineLayoutHandle::invalid(),
            options: Vec::new(),
            instances: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct ShaderInstance {
    pub stages: Vec<ShaderStage>,
    pub keys: Vec<usize>,
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

    pub fn set_path(mut self, path: &str) -> Self {
        self.product.path = path.to_owned();
        self
    }

    pub fn set_pipeline_layout(mut self, pipeline_layout: &str) -> Result<Self> {
        let pl_handle = self
            .mdl
            .get_object_handle::<PipelineLayout>(pipeline_layout)
            .ok_or_else(|| anyhow!("Unknown PipelineLayout '{pipeline_layout}'"))?;

        self.product.pipeline_layout = pl_handle;

        Ok(self)
    }

    pub fn add_option(mut self, define: &str) -> Self {
        if !self.define_set.contains(define) {
            self.define_set.insert(define.to_owned());
            self.product.options.push(define.to_owned());
        }
        self
    }

    pub fn add_instance(mut self, option_list: &[String], stages: &[String]) -> Result<Self> {
        let mut index_list = Vec::new();

        for (_, option) in option_list.iter().enumerate() {
            let pos = self.product.options.iter().position(|x| x == option);
            if pos.is_none() {
                return Err(anyhow!("Invalid option '{option}'"));
            }
            index_list.push(pos.unwrap());
        }

        index_list.sort_unstable();

        self.product.instances.push(ShaderInstance {
            stages: Self::stages_from_string_list(stages)?,
            keys: index_list,
        });

        Ok(self)
    }

    fn stages_from_string_list(stages: &[String]) -> Result<Vec<ShaderStage>> {
        let mut result = Vec::new();

        for stage in stages {
            let stage = match stage.as_str() {
                "VS" => ShaderStage::Vertex,
                "PS" => ShaderStage::Fragment,
                "CS" => ShaderStage::Compute,
                _ => {
                    return Err(anyhow!("Invalid shader stage '{stage}'"));
                }
            };
            result.push(stage);
        }

        if result.is_empty() {
            return Err(anyhow!("No shader stages specified"));
        }

        let mut use_graphic_pipeline = false;
        let mut use_compute_pipeline = false;

        for stage in &result {
            match stage {
                ShaderStage::Vertex | ShaderStage::Fragment => use_graphic_pipeline = true,
                ShaderStage::Compute => use_compute_pipeline = true,
            }
        }

        if use_graphic_pipeline && use_compute_pipeline {
            return Err(anyhow!("Conflicting shader stages"));
        }

        Ok(result)
    }

    pub fn build(self) -> Shader {
        self.product
    }
}
