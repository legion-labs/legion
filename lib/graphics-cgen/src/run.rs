use std::{env, path::{PathBuf}};

use anyhow::{anyhow, Result};
use log::info;
use path_clean::PathClean;

use crate::{
    generators::{self, Generator, GeneratorContext, Product},
    parser::{from_syn, from_yaml},
};

pub struct CGenContext {
    root_file: PathBuf,
    output_folder: PathBuf,
}

impl Default for CGenContext {
    fn default() -> Self {
        Self {
            root_file: PathBuf::default(),
            output_folder: PathBuf::default(),
        }
    }
}


fn str_to_abspath(path: &str) -> PathBuf {    
    let path =
    { 
        if std::path::MAIN_SEPARATOR == '\\' {
            path.replace("/", "\\")
        }
        else {
            path.replace("\\", "/")
        }
    };
    let mut path = PathBuf::from(path);
    if path.is_relative() {
        let mut curpath = env::current_dir().unwrap();
        curpath.push(path);            
        path = curpath;
    } 
    path.clean()
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
        
        let path = str_to_abspath(root_file);

        if !path.exists() || !path.is_file() {
            return Err( anyhow!("The file {} does not exist ", root_file  ) );
        }

        self.context.root_file = path;

        Ok(self)
    }

    pub fn set_output_folder(&mut self, output_folder: &str) -> Result<&mut Self> {
        let output_folder = str_to_abspath(output_folder);
        self.context.output_folder = output_folder;
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

fn run_internal(context: &CGenContext) -> Result<()> {
    //
    // Load model
    //
    info!("Load model from {}", context.root_file.display());

    let root_file_ext = context
        .root_file
        .extension()
        .ok_or(anyhow!("No extension"))?;

    let model = match root_file_ext.to_str().unwrap() {
        "yaml" => from_yaml(&context.root_file)?,
        "rs" => from_syn(&context.root_file)?,
        _ => return Err(anyhow!("Unknown extension")),
    };

    //
    // Prepare generation context
    //
    let mut hlsl_folder = context.output_folder.clone();
    let mut rust_folder = context.output_folder.clone();
    hlsl_folder.push("hlsl");
    rust_folder.push("rust");

    let gen_context = GeneratorContext::new(&model, hlsl_folder, rust_folder);

    info!("{}", gen_context);

    //
    // generation step
    //
    let mut generators: Vec<Box<&dyn Generator>> = Vec::new();
    let generator = generators::hlsl::type_generator::TypeGenerator::default();
    generators.push(Box::new(&generator));
    let generator = generators::hlsl::pipelinelayout_generator::PipelineLayoutGenerator::default();
    generators.push(Box::new(&generator));

    let mut products = Vec::<Product>::new();
    for generator in generators {                
        let mut pr = generator.run(&gen_context);
        products.append(&mut pr);
    }

    //
    // write to disk
    //
    for product in &products {
        product.write_to_disk()?;
    }

    Ok(())
}
