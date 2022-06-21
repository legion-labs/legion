//! Api Codegen crate used in Node applications

use std::{collections::HashMap, path::PathBuf};

use lgn_api_codegen::{Language, TypeScriptAliasMappings, TypeScriptOptions};
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
    /// Aliases for external API references
    pub alias_mappings: Option<HashMap<String, String>>,
    /// Filename without prefix nor extension
    pub filename: String,
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
            alias_mappings: options
                .alias_mappings
                .unwrap_or_default()
                .try_into()
                .map_err(|err: lgn_api_codegen::errors::Error| {
                    Error::from_reason(err.to_string())
                })?,
            prettier_config_path: options.prettier_config_path.map(PathBuf::from),
            with_package_json: options.with_package_json.unwrap_or_default(),
            skip_format: options.skip_format.unwrap_or_default(),
            filename: options.filename,
        }),
        &options.path,
        &options.api_names,
        &options.out_dir,
    ) {
        return Err(Error::from_reason(format!("{}", error)));
    }

    Ok(())
}

#[derive(Debug)]
#[napi(object)]
pub struct ApiOption {
    /// Path to the _folder_ that contains all the apis definition files (.yaml), prefer absolute paths
    pub path: String,
    /// Name(s) of the api to generate the client code for
    pub names: Vec<String>,
    /// Filename without prefix nor extension
    pub filename: String,
}

#[derive(Debug)]
#[napi(object)]
pub struct GenerateAllOption {
    /// Output directory, prefer absolute paths
    pub out_dir: String,
    /// Path to a Prettier config file
    pub prettier_config_path: Option<String>,
    /// Generates a full-fledged node module including a `package.json` file
    pub with_package_json: Option<bool>,
    /// Skips code formatting
    pub skip_format: Option<bool>,
    /// Aliases for external API references
    pub alias_mappings: Option<HashMap<String, String>>,
    /// Api options
    pub api_options: Vec<ApiOption>,
}

/// Generate all api clients.
///
/// # Errors
///
/// Throws if the generation fails (file not found, invalid, etc...)
#[allow(clippy::needless_pass_by_value)]
#[napi]
pub fn generate_all(options: GenerateAllOption) -> Result<()> {
    let prettier_config_path = options.prettier_config_path.map(PathBuf::from);

    let alias_mappings: TypeScriptAliasMappings = options
        .alias_mappings
        .unwrap_or_default()
        .try_into()
        .map_err(|err: lgn_api_codegen::errors::Error| Error::from_reason(err.to_string()))?;

    let result = options
        .api_options
        .into_iter()
        .map(|api| {
            lgn_api_codegen::generate(
                Language::TypeScript(TypeScriptOptions {
                    alias_mappings: alias_mappings.clone(),
                    filename: api.filename,
                    with_package_json: options.with_package_json.unwrap_or_default(),
                    skip_format: options.skip_format.unwrap_or_default(),
                    prettier_config_path: prettier_config_path.clone(),
                }),
                &api.path,
                api.names,
                &options.out_dir,
            )
        })
        .collect::<lgn_api_codegen::errors::Result<Vec<Vec<PathBuf>>, _>>();

    if let Err(error) = result {
        return Err(Error::from_reason(error.to_string()));
    }

    Ok(())
}
