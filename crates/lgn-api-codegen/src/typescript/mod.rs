use std::{
    fs::{read_to_string, remove_dir_all, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::anyhow;
use askama::Template;
use serde::Deserialize;
use tempfile::tempdir;
use which::which;

use crate::{
    api_types::{GenerationContext, MediaType, Type},
    errors::Error,
    Language, Result, TypeScriptOptions,
};

mod filters;

#[derive(askama::Template)]
#[template(path = "index.ts.jinja", escape = "none")]
struct TypeScriptIndexTemplate;

#[derive(askama::Template)]
#[template(path = "api.ts.jinja", escape = "none")]
struct TypeScriptApiTemplate<'a> {
    pub ctx: &'a TypeScriptGenerationContext,
}

#[derive(askama::Template)]
#[template(path = "package.json.jinja", escape = "none")]
struct TypeScriptPackageTemplate;

pub type TypeScriptGenerationContext = GenerationContext<TypeScriptOptions>;

#[derive(Debug, Deserialize)]
struct NpmConfig {
    cache: PathBuf,
}

impl NpmConfig {
    fn read() -> Result<NpmConfig> {
        let npm_path = which("npm").map_err(|_err| {
            Error::TypeScriptFormat(anyhow!(
                "couldn't find npm binary, is it installed and accessible in the path?"
            ))
        })?;

        let mut cmd = Command::new(npm_path);

        cmd.args(["config", "list", "--json"]);

        let output = cmd.output().map_err(|error| {
            Error::TypeScriptFormat(anyhow!("npm config command was not successful: {}", error))
        })?;

        let config: NpmConfig = serde_json::from_slice(&output.stdout).map_err(|_err| {
            Error::TypeScriptFormat(anyhow!("npm config list output is not valid"))
        })?;

        Ok(config)
    }
}

/// Removes the npx cache folder _entirely_.
fn clean_npx_cache() -> Result<bool> {
    let npm_config = NpmConfig::read()?;

    let npx_cache_path = npm_config.cache.join("_npx");

    if npx_cache_path.exists() {
        remove_dir_all(npx_cache_path)
            .map_err(|_err| Error::TypeScriptFormat(anyhow!("couldn't clean npm cache")))?;

        return Ok(true);
    }

    Ok(false)
}

/// Executes [Prettier](https://prettier.io/) under the hood to format the provided content on the fly.
///
/// # Errors
///
/// Can fail if `npm` or `npx` are not installed, of if Prettier returns a non-zero code.
fn format_typescript(
    options: &TypeScriptOptions,
    content: &str,
    temp_file_name: &str,
) -> Result<String> {
    let prettier_config_path = options
        .prettier_config_path
        .clone()
        .map(|path| path.to_string_lossy().into_owned());

    let dir =
        tempdir().map_err(|_err| Error::TypeScriptFormat(anyhow!("couldn't create a tmp dir")))?;

    let file_path = dir.path().join(temp_file_name);

    let mut file = File::create(&file_path)?;

    write!(file, "{}", content)
        .map_err(|_err| Error::TypeScriptFormat(anyhow!("couldn't write content to tmp file")))?;

    let npx_path = which("npx").map_err(|_err| {
        Error::TypeScriptFormat(anyhow!(
            "couldn't find npx binary, is it installed and accessible in the path?"
        ))
    })?;

    let mut cmd = Command::new(npx_path);

    let tmp_file_path = file_path.as_path().to_string_lossy();

    let mut args = vec![
        "--yes",
        "prettier",
        "--loglevel",
        "silent",
        "--write",
        &tmp_file_path,
    ];

    match prettier_config_path {
        None => args.push("--no-config"),
        Some(ref path) => {
            args.push("--config");
            args.push(path);
        }
    };

    cmd.args(&args);

    let status = cmd.status().map_err(|error| {
        Error::TypeScriptFormat(anyhow!(
            "npx prettier command was not successful: {}",
            error
        ))
    })?;

    if status.success() {
        let formatted_content = read_to_string(&file_path)
            .map_err(|_err| Error::TypeScriptFormat(anyhow!("couldn't read tmp file")))?;

        Ok(formatted_content)
    } else {
        Err(Error::TypeScriptFormat(anyhow!(
            "npx prettier command was not successful: {}",
            status
        )))
    }
}

/// Safe-ish alternative to [`format_typescript`] that cleans the NPM cache
/// and tries again if the format fails the first time
///
/// # Errors
///
/// Can fail if `npm` or `npx` are not installed, of if Prettier returns a non-zero code.
fn safe_format_typescript(
    options: &TypeScriptOptions,
    content: &str,
    temp_file_name: &str,
) -> Result<String> {
    format_typescript(options, content, temp_file_name).or_else(|_err| {
        let _npm_cache_cleaned = clean_npx_cache()?;
        format_typescript(options, content, temp_file_name)
    })
}

fn generate_index_content(ctx: &TypeScriptGenerationContext) -> Result<String> {
    let mut content = TypeScriptIndexTemplate.render()?;

    if !ctx.options.skip_format {
        content = safe_format_typescript(&ctx.options, &content, "index.ts")?;
    }

    Ok(content)
}

fn generate_api_content(ctx: &TypeScriptGenerationContext) -> Result<String> {
    let mut content = TypeScriptApiTemplate { ctx }.render()?;

    if !ctx.options.skip_format {
        content = safe_format_typescript(&ctx.options, &content, "index.ts")?;
    }

    Ok(content)
}

fn generate_package_content(options: &TypeScriptOptions) -> Result<String> {
    let mut content = TypeScriptPackageTemplate.render()?;

    if !options.skip_format {
        content = safe_format_typescript(options, &content, "package.json")?;
    }

    Ok(content)
}

impl Language {
    pub(crate) fn generate_typescript(
        ctx: GenerationContext,
        options: TypeScriptOptions,
        output_dir: &Path,
    ) -> Result<()> {
        if &options.filename == "index" {
            return Err(Error::TypeScriptFilename);
        }

        std::fs::create_dir_all(output_dir)?;

        let ctx = ctx.with_options(options);

        let output_file = output_dir.join(&ctx.options.filename).with_extension("ts");
        let content = generate_api_content(&ctx)?;
        std::fs::write(output_file, content)?;

        let output_file = output_dir.join("index").with_extension("ts");
        let content = generate_index_content(&ctx)?;
        std::fs::write(output_file, content)?;

        if ctx.options.with_package_json {
            let output_file = output_dir.join("package.json");
            let content = generate_package_content(&ctx.options)?;

            std::fs::write(output_file, content)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{openapi_loader::OpenApiRefLocation, visitor::Visitor, OpenApiLoader};

    use super::*;

    #[test]
    fn test_ts_index_generation() {
        let loader = OpenApiLoader::default();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/api-codegen")
            .canonicalize()
            .unwrap();
        let openapi = loader
            .load_openapi(OpenApiRefLocation::new(&root, "cars.yaml".into()))
            .unwrap();
        let ctx = Visitor::new(root)
            .visit(&[openapi.clone()])
            .unwrap()
            .with_options(TypeScriptOptions::default());
        let content = generate_index_content(&ctx).unwrap();

        insta::assert_snapshot!(content);
    }

    #[test]
    fn test_ts_api_generation() {
        let loader = OpenApiLoader::default();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/api-codegen")
            .canonicalize()
            .unwrap();
        let openapi = loader
            .load_openapi(OpenApiRefLocation::new(&root, "cars.yaml".into()))
            .unwrap();
        let ctx = Visitor::new(root)
            .visit(&[openapi.clone()])
            .unwrap()
            .with_options(TypeScriptOptions::default());
        let content = generate_api_content(&ctx).unwrap();

        insta::assert_snapshot!(content);
    }

    #[test]
    fn test_ts_package_generation() {
        let content = generate_package_content(&TypeScriptOptions::default()).unwrap();

        insta::assert_snapshot!(content);
    }
}
