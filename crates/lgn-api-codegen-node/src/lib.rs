//! Api Codegen crate used in Node applications

use std::path::PathBuf;

<<<<<<< HEAD
use lgn_api_codegen::{Language, TypeScriptOptions};
=======
use lgn_api_codegen::{GenerationOptions, Language};
>>>>>>> f25890514 (Codegen: Optionally generate node modules when targetting TS)
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
    /// Path to a Prettier config file
    pub prettier_config_path: Option<String>,
    /// Generates a full-fledged node module including a `package.json` file
    pub with_package_json: Option<bool>,
    /// Skips code formatting
    pub skip_format: Option<bool>,
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
        Language::TypeScript(TypeScriptOptions {
            prettier_config_path: options.prettier_config_path.map(PathBuf::from),
            with_package_json: options.with_package_json.unwrap_or_default(),
            skip_format: options.skip_format.unwrap_or_default(),
        }),
        &options.path,
        &options.api_names,
        &options.out_dir,
    ) {
        return Err(Error::from_reason(format!("{}", error)));
    }

    Ok(())
}
