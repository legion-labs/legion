use std::{
    env::{self},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{anyhow, Result};
use log::info;
use relative_path::RelativePath;

use crate::{
    generators::{self, product::Product, GeneratorContext},
    parser::{from_syn, from_yaml},
};

pub struct CGenContext {
    pub(super) root_file: PathBuf,
    pub(super) outdir_hlsl: PathBuf,
    pub(super) outdir_rust: PathBuf,
}

impl Default for CGenContext {
    fn default() -> Self {
        let cur_dir = env::current_dir().unwrap();
        Self {
            root_file: RelativePath::new("root.cgen").to_path(&cur_dir),
            outdir_hlsl: RelativePath::new("generated_hlsl").to_path(&cur_dir),
            outdir_rust: RelativePath::new("generated_rust").to_path(&cur_dir),
        }
    }
}

pub struct CGenContextBuilder {
    context: CGenContext,
}

impl CGenContextBuilder {
    pub fn new() -> Self {
        Self {
            context: CGenContext::default(),
        }
    }

    pub fn set_root_file(&mut self, root_file: &str) -> Result<&mut Self> {
        let abs_path = to_abs_path(root_file)?;
        if !abs_path.exists() || !abs_path.is_file() {
            return Err(anyhow!("File {} does not exist ", root_file));
        }
        self.context.root_file = abs_path;

        Ok(self)
    }

    pub fn set_outdir_hlsl(&mut self, outdir: &str) -> Result<&mut Self> {
        self.context.outdir_hlsl = to_abs_path(outdir)?;

        Ok(self)
    }

    pub fn set_outdir_rust(&mut self, outdir: &str) -> Result<&mut Self> {
        self.context.outdir_rust = to_abs_path(outdir)?;

        Ok(self)
    }

    pub fn build(self) -> CGenContext {
        self.context
    }
}

pub fn run(context: &CGenContext) -> Result<()> {
    // timing
    run_internal(context)
}

fn to_abs_path(path: &str) -> Result<PathBuf> {
    let outdir_path = Path::new(path);

    Ok(if outdir_path.is_relative() {
        let cur_dir = env::current_dir()?;
        RelativePath::new(path).to_logical_path(cur_dir)
    } else {
        outdir_path.to_path_buf()
    })
}

fn run_internal(context: &CGenContext) -> Result<()> {
    //
    // Load model
    //
    info!("Load model from {}", context.root_file.display());

    let root_file_ext = context.root_file.extension().ok_or(anyhow!(
        "No extension on root file {}",
        context.root_file.display()
    ))?;

    let model = match root_file_ext.to_str().unwrap() {
        "yaml" => Arc::new(from_yaml(&context.root_file)?),
        "cgen" => Arc::new(from_syn(&context.root_file)?),
        _ => return Err(anyhow!("Unknown extension")),
    };

    //
    // Prepare generation context
    //

    let gen_context = GeneratorContext::new(&model, context);

    //
    // generation step
    //
    let mut generators = Vec::<generators::GeneratorFunc>::new();
    generators.push(generators::hlsl::type_generator::run);
    generators.push(generators::hlsl::pipelinelayout_generator::run);
    generators.push(generators::rust::base_mod_generator::run);
    generators.push(generators::rust::type_generator::run);

    let mut products = Vec::<Product>::new();
    for generator in generators {
        let mut pr = generator(&gen_context);
        products.append(&mut pr);
    }

    //
    // write to disk
    //
    for product in &products {
        product.write_to_disk(&gen_context)?;
    }

    Ok(())
}
