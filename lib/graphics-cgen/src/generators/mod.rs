mod file_writer;
pub mod hlsl;
pub mod rust;

use std::{collections::HashSet, fmt::Display, io::Write, path::PathBuf};

use anyhow::Result;

use crate::model::{CGenType, Model, ModelKey, PipelineLayout};

pub enum CGenVariant {
    Hlsl,
    Rust,
}

pub struct GeneratorContext<'a> {
    model: &'a Model,
    hlsl_folder: PathBuf,
    rust_folder: PathBuf,
}

impl<'a> Display for GeneratorContext<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("GeneratorContext:\n")?;
        f.write_str("Output folders:\n")?;
        f.write_str(&format!("* HLSL folder: {}\n", self.hlsl_folder.display()))?;
        f.write_str(&format!("* Rust folder: {}\n", self.rust_folder.display()))?;

        Ok(())
    }
}

impl<'a> GeneratorContext<'a> {
    pub fn new(model: &'a Model, hlsl_folder: PathBuf, rust_folder: PathBuf) -> Self {
        Self {
            model,
            hlsl_folder,
            rust_folder,
        }
    }

    fn get_base_folder(&self, cgen_variant: CGenVariant) -> PathBuf {
        match cgen_variant {
            CGenVariant::Hlsl => self.hlsl_folder.clone(),
            CGenVariant::Rust => self.rust_folder.clone(),
        }
    }

    fn get_type_abspath(&self, ty: &CGenType, cgen_variant: CGenVariant) -> PathBuf {
        let mut path = self.get_base_folder(cgen_variant);
        path.push("types");
        match ty {
            CGenType::Struct(s) => {
                path.push(&s.name);
            }
            CGenType::Native(_) => panic!()
        }
        path.set_extension("hlsl");

        path
    }

    fn get_pipelinelayout_abspath(
        &self,
        pipeline_layout: &PipelineLayout,
        cgen_variant: CGenVariant,
    ) -> PathBuf {
        let mut path = self.get_base_folder(cgen_variant);
        path.push("pipelinelayout");
        path.push(&pipeline_layout.name);
        path.set_extension("rs");

        path
    }

    pub fn get_type_dependencies(&self, ty: &CGenType) -> Result<HashSet<ModelKey>> {
        let mut set = HashSet::new();

        match ty {
            CGenType::Native(_) => (),
            CGenType::Struct(st) => {
                for mb in st.members.iter() {
                    let mb_type = self.model.get::<CGenType>(mb.type_key).unwrap();
                    match mb_type {
                        CGenType::Native(_) => (),
                        CGenType::Struct(_) => {
                            set.insert(mb.type_key);
                        }
                    }
                }
            }
        }

        Ok(set)
    }
}

pub trait Generator {
    fn run(&self, ctx: &GeneratorContext<'_>) -> Vec<Product>;
}

#[derive(Debug)]
pub struct Product {
    path: PathBuf,
    content: String,
}

impl Product {
    pub fn write_to_disk(&self) -> Result<()> {
        let mut dir_builder = std::fs::DirBuilder::new();
        dir_builder.recursive(true);
        dir_builder.create(&self.path.parent().unwrap())?;

        let file_content = self.content.to_string();

        let mut output = std::fs::File::create(&self.path)?;
        output.write(&file_content.as_bytes())?;

        Ok(())
    }
}
