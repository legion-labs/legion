//! Legion `OpenApi` code generator crate.
//!
//! Provides code generation for various languages based on an `OpenAPI` v3 specification.
//!

// crate-specific lint exceptions:
//#![allow()]

pub(crate) mod api_types;
pub(crate) mod errors;
pub(crate) mod filters;
pub(crate) mod openapi_loader;
pub(crate) mod rust;
pub(crate) mod typescript;
pub(crate) mod visitor;

use api_types::GenerationContext;
use clap::ArgEnum;
use errors::{Error, Result};
use openapi_loader::{OpenApi, OpenApiElement, OpenApiLoader, OpenApiRefLocation};
use rust::RustGenerator;
use std::path::{Path, PathBuf};
use typescript::TypeScriptGenerator;
use visitor::Visitor;

#[derive(Debug, Copy, Clone, ArgEnum)]
pub enum Language {
    Rust,
    #[clap(name = "typescript")]
    TypeScript,
}

/// Generates the code for the specificed language and the specified APIs.
///
/// The code will look for each API using the specified name suffixed by `.yaml`
/// in the specified root.
///
/// If the root is a relative file, it will be resolved relative to the current
/// working directory.
///
/// # Errors
///
/// If the generation fails to complete.
pub fn generate(
    language: Language,
    root: impl AsRef<Path>,
    openapis: impl IntoIterator<Item = impl AsRef<str>>,
    output_dir: impl AsRef<Path>,
) -> Result<Vec<PathBuf>> {
    let generator = load_generator_for_language(language);

    let root = if root.as_ref().is_relative() {
        std::env::current_dir()?.join(root)
    } else {
        root.as_ref().to_path_buf()
    }
    .canonicalize()?;

    let loader = OpenApiLoader::default();

    let openapis = openapis
        .into_iter()
        .map(|name| {
            let ref_loc =
                OpenApiRefLocation::new(&root, PathBuf::from(format!("{}.yaml", name.as_ref())));

            loader.load_openapi(ref_loc)
        })
        .collect::<Result<Vec<_>>>()?;

    let ctx = Visitor::new(root).visit(&openapis)?;

    generator.generate(&ctx, output_dir.as_ref())?;

    Ok(loader.get_all_files())
}

#[macro_export]
macro_rules! generate {
    ($language:ident, $root:expr, $openapis:expr) => {{
        match lgn_api_codegen::generate(
            lgn_api_codegen::Language::$language,
            $root,
            $openapis,
            std::env::var("OUT_DIR")?,
        ) {
            Ok(files) => {
                for file in files {
                    println!("cargo:rerun-if-changed={}", file.display());
                }

                Ok(())
            }
            Err(err) => Err(err),
        }
    }};
}

pub(crate) trait Generator {
    fn generate(&self, ctx: &GenerationContext, output_dir: &Path) -> Result<()>;
}

fn load_generator_for_language(language: Language) -> Box<dyn Generator> {
    match language {
        Language::Rust => Box::new(RustGenerator::default()),
        Language::TypeScript => Box::new(TypeScriptGenerator::default()),
    }
}
