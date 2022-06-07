//! Api Codegen crate used in Node applications

use lgn_api_codegen::Language;
use napi::bindgen_prelude::{Error, Result};
use napi_derive::napi;

#[derive(Debug)]
#[napi(object)]
pub struct GenerateOption {
    /// Path to the _folder_ that contains all the apis definition files (.yaml), prefer absolute paths
    pub path: String,
    /// Name(s) of the api to generate the client code for
    pub api_names: Vec<String>,
    /// Output directory, prefer absolute paths
    pub out_dir: String,
}

/// Generate api clients.
///
/// # Errors
///
/// Throws if the generation fails (file not found, invalid, etc...)
#[allow(clippy::needless_pass_by_value)]
#[napi]
pub fn generate(options: GenerateOption) -> Result<()> {
    if let Err(error) = lgn_api_codegen::generate(
        Language::TypeScript,
        lgn_api_codegen::GenerationOptions::default(),
        &options.path,
        &options.api_names,
        &options.out_dir,
    ) {
        return Err(Error::from_reason(format!("{}", error)));
    }

    Ok(())
}
