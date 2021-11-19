mod file_writer;
pub mod hlsl;
pub mod product;
pub mod rust;

use std::{collections::HashSet, ops::Add, path::Path};

use anyhow::Result;
use heck::SnakeCase;
use relative_path::{RelativePath, RelativePathBuf};

use crate::{
    model::{CGenType, Model, ModelKey},
    run::CGenContext,
};

use self::product::Product;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CGenVariant {
    Hlsl,
    Rust,
}

pub type GeneratorFunc = for<'r, 's> fn(&'r GeneratorContext<'s>) -> Vec<Product>;
pub struct GeneratorContext<'a> {
    model: &'a Model,
    cgen_context: &'a CGenContext,
}

impl<'a> GeneratorContext<'a> {
    pub fn new(model: &'a Model, cgen_context: &'a CGenContext) -> Self {
        Self {
            model,
            cgen_context,
        }
    }

    fn get_base_folder(&self, cgen_variant: CGenVariant) -> &Path {
        match cgen_variant {
            CGenVariant::Hlsl => &self.cgen_context.outdir_hlsl,
            CGenVariant::Rust => &self.cgen_context.outdir_rust,
        }
    }

    fn get_file_ext(cgen_variant: CGenVariant) -> &'static str {
        match cgen_variant {
            CGenVariant::Hlsl => "hlsl",
            CGenVariant::Rust => "rs",
        }
    }

    fn get_type_folder(&self) -> &RelativePath {
        RelativePath::new("types")
    }

    fn get_rel_type_path(&self, ty: &CGenType, cgen_variant: CGenVariant) -> RelativePathBuf {
        let mut rel_path = self.get_type_folder().to_relative_path_buf();
        match ty {
            CGenType::Struct(_) => {
                rel_path.push(Self::get_type_filename(ty, cgen_variant));
            }
            CGenType::Native(_) => panic!(),
        }
        rel_path
    }

    fn get_type_filename(ty: &CGenType, cgen_variant: CGenVariant) -> String {
        let result = match ty {
            CGenType::Native(_) => panic!("Not possible"),
            CGenType::Struct(st) => st
                .name
                .to_snake_case()
                .add(".")
                .add(Self::get_file_ext(cgen_variant)),
        };
        result
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
