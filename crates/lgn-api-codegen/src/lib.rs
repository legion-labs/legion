//! Legion `OpenApi` code generator crate.
//!
//! Provides code generation for various languages based on an `OpenAPI` v3 specification.
//!

// crate-specific lint exceptions:
//#![allow()]

pub(crate) mod api;
pub(crate) mod errors;
pub(crate) mod filters;
pub(crate) mod openapi_ext;
pub(crate) mod rust;
pub(crate) mod visitor;

use api::Api;
use errors::{Error, Result};
use rust::RustGenerator;
use std::path::Path;

/// Generates the code for the specificed language.
///
/// # Errors
///
/// If the generation fails to complete.
pub fn generate(
    language: &str,
    openapi_file: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> Result<()> {
    let openapi_file = std::fs::File::open(openapi_file)?;
    let openapi: openapiv3::OpenAPI = serde_yaml::from_reader(&openapi_file)?;
    let api = Api::try_from(&openapi)?;

    let generator = load_generator_for_language(language)?;
    generator.generate(&api, output_dir.as_ref())
}

pub(crate) trait Generator {
    fn generate(&self, api: &Api, output_dir: &Path) -> Result<()>;
}

fn load_generator_for_language(language: &str) -> Result<Box<dyn Generator>> {
    Ok(match language {
        "rust" => Box::new(RustGenerator::default()),
        _ => return Err(Error::Unsupported(format!("language: {}", language))),
    })
}
