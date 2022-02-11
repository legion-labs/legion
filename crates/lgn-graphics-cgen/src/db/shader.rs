use std::collections::HashSet;

use anyhow::{anyhow, Result};
use lgn_graphics_api::ShaderStageFlags;

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
    pub stages: ShaderStageFlags,
    pub key: Vec<u8>,
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

    pub fn add_instance(mut self, option_list: &Vec<String>, stages: &Vec<String>) -> Result<Self> {
        let mut index_list = Vec::new();

        for (i, option) in option_list.iter().enumerate() {
            if !self.product.options.contains(option) {
                return Err(anyhow!("Invalid option '{option}'"));
            }
            index_list.push(u8::try_from(i).unwrap());
        }

        self.product.instances.push(ShaderInstance {
            stages: Self::stages_from_string_list(stages)?,
            key: index_list,
        });

        Ok(self)
    }

    fn stages_from_string_list(stages: &Vec<String>) -> Result<ShaderStageFlags> {
        let mut result = ShaderStageFlags::NONE;

        for stage in stages {
            match stage.as_str() {
                "VS" => result |= ShaderStageFlags::VERTEX_FLAG,
                "PS" => result |= ShaderStageFlags::FRAGMENT_FLAG,
                "CS" => result |= ShaderStageFlags::COMPUTE_FLAG,
                _ => {
                    return Err(anyhow!("Invalid shader stage '{stage}'"));
                }
            }

            let use_graphic_pipeline = (result
                & (ShaderStageFlags::VERTEX_FLAG | ShaderStageFlags::FRAGMENT_FLAG))
                != ShaderStageFlags::NONE;

            let use_compute_pipeline =
                (result & ShaderStageFlags::COMPUTE_FLAG) != ShaderStageFlags::NONE;

            if use_graphic_pipeline && use_compute_pipeline {
                return Err(anyhow!("Conflicting shader stages '{result}'"));
            }
        }

        println!("{}", result);

        Ok(result)
    }

    pub fn build(self) -> Shader {
        self.product
    }
}
