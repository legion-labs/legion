use std::{
    env::{self},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use lgn_telemetry::info;
use relative_path::RelativePath;

use crate::{
    generators::{self, product::Product, GeneratorContext},
    parser::from_syn,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CGenVariant {
    Hlsl,
    Rust,
    Blob,
}

pub struct CGenContext {
    pub(super) root_file: PathBuf,
    pub(super) outdir: PathBuf,
}

impl Default for CGenContext {
    fn default() -> Self {
        let cur_dir = env::current_dir().unwrap();
        Self {
            root_file: RelativePath::new("root.cgen").to_path(&cur_dir),
            outdir: RelativePath::new("cgen_out").to_path(&cur_dir),
        }
    }
}

impl CGenContext {
    pub fn out_dir(&self, variant: CGenVariant) -> PathBuf {
        match variant {
            CGenVariant::Hlsl => RelativePath::new("hlsl").to_path(&self.outdir),
            CGenVariant::Rust => RelativePath::new("rust").to_path(&self.outdir),
            CGenVariant::Blob => RelativePath::new("blob").to_path(&self.outdir),
        }
    }
}

pub struct CGenBuildResult {
    pub input_dependencies: Vec<PathBuf>,
}

#[derive(Default)]
pub struct CGenContextBuilder {
    context: CGenContext,
}

impl CGenContextBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set root file
    ///
    /// # Errors
    /// File does not exists or invalid path..
    pub fn set_root_file(&mut self, root_file: &impl AsRef<Path>) -> Result<()> {
        let abs_path = to_abs_path(root_file)?;
        if !abs_path.exists() || !abs_path.is_file() {
            return Err(anyhow!(
                "File {} does not exist ",
                root_file.as_ref().display()
            ));
        }
        self.context.root_file = abs_path;

        Ok(())
    }
    /// Set output directory
    ///
    /// # Errors
    /// Invalid path.
    pub fn set_outdir(&mut self, outdir: &impl AsRef<Path>) -> Result<()> {
        self.context.outdir = to_abs_path(outdir)?;

        Ok(())
    }

    pub fn build(self) -> CGenContext {
        self.context
    }
}

/// Run code generation
///
/// # Errors
/// Returns an error.
pub fn run(context: &CGenContext) -> Result<CGenBuildResult> {
    // todo: timing
    run_internal(context)
}

fn to_abs_path(path: &impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref();
    Ok(if path.is_relative() {
        let cur_dir = env::current_dir()?;
        RelativePath::from_path(path)?.to_logical_path(cur_dir)
    } else {
        path.to_path_buf()
    })
}

fn run_internal(context: &CGenContext) -> Result<CGenBuildResult> {
    //
    // Load model
    //
    info!("Load model from {}", context.root_file.display());

    let root_file_ext = context
        .root_file
        .extension()
        .ok_or_else(|| anyhow!("No extension on root file {}", context.root_file.display()))?;

    let parsing_result = match root_file_ext.to_str().unwrap() {
        "cgen" => from_syn(&context.root_file)?,
        _ => return Err(anyhow!("Unknown extension")),
    };
    let model = &parsing_result.model;

    //
    // generation step
    //
    let gen_context = GeneratorContext::new(model);
    let generators = [
        generators::hlsl::type_generator::run,
        generators::hlsl::descriptorset_generator::run,
        generators::hlsl::pipelinelayout_generator::run,
        generators::rust::base_mod_generator::run,
        generators::rust::type_generator::run,
        generators::rust::descriptorset_generator::run,
        generators::rust::pipelinelayout_generator::run,
        generators::rust::cgen_def_generator::run,
    ];
    let mut products = Vec::<Product>::new();
    for generator in generators {
        let mut pr = generator(&gen_context);
        products.append(&mut pr);
    }

    //
    // write to disk
    //
    for product in &products {
        product.write_to_disk(context)?;
    }

    // done
    Ok(CGenBuildResult {
        input_dependencies: parsing_result.input_dependencies,
    })
}
