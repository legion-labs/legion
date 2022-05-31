//! Legion `OpenApi` code generator crate.
//!
//! Provides code generation for various languages based on an `OpenAPI` v3 specification.
//!

// crate-specific lint exceptions:
//#![allow()]

pub(crate) mod api;
pub(crate) mod errors;
pub(crate) mod filters;
pub(crate) mod openapi_loader;
pub(crate) mod openapi_path;
pub(crate) mod rust;
pub(crate) mod typescript;
pub(crate) mod visitor;

use api::Api;
use clap::ArgEnum;
use errors::{Error, Result};
use openapi_loader::{OpenApi, OpenApiElement, OpenApiLoader};
use openapi_path::OpenAPIPath;
use rust::RustGenerator;
use std::path::Path;
use typescript::TypeScriptGenerator;

#[derive(Debug, Copy, Clone, ArgEnum)]
pub enum Language {
    Rust,
    TypeScript,
}

/// Generates the code for the specificed language.
///
/// # Errors
///
/// If the generation fails to complete.
pub fn generate(
    language: Language,
    openapi_file: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> Result<()> {
    let openapi_file = openapi_file.as_ref();
    let loader = OpenApiLoader::default();
    let openapi = loader.load_openapi(openapi_file.to_path_buf().into())?;
    let api = openapi.try_into()?;

    let generator = load_generator_for_language(language);
    generator.generate(&api, openapi_file, output_dir.as_ref())
}

#[macro_export]
macro_rules! generate {
    ($language:ident, $apis_path:expr, $api_name:ident, $($api_names:ident),+) => {{
        lgn_api_codegen::generate!($language, $apis_path, $($api_names),+)?;
        lgn_api_codegen::generate!($language, $apis_path, $api_name)
    }};
    ($language:ident, $apis_path:expr, $api_name:ident) => {{
        let openapi_file =
            std::path::PathBuf::from($apis_path).join(concat!(stringify!($api_name), ".yaml"));

        println!("cargo:rerun-if-changed={}", openapi_file.display());

        lgn_api_codegen::generate(
            lgn_api_codegen::Language::$language,
            openapi_file,
            std::env::var("OUT_DIR")?,
        )
    }};
}

pub(crate) trait Generator {
    fn generate(&self, api: &Api, openapi_file: &Path, output_dir: &Path) -> Result<()>;
}

fn load_generator_for_language(language: Language) -> Box<dyn Generator> {
    match language {
        Language::Rust => Box::new(RustGenerator::default()),
        Language::TypeScript => Box::new(TypeScriptGenerator::default()),
    }
}
