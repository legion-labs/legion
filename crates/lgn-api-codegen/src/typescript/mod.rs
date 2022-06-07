use std::{
    borrow::Cow,
    fs::{read_to_string, File},
    io::Write,
    path::Path,
    process::Command,
};

use anyhow::anyhow;
use askama::Template;
use tempfile::tempdir;
use which::which;

use crate::{
    api_types::{GenerationContext, MediaType, Type},
    errors::Error,
    Generator, Result,
};

mod filters;

#[derive(askama::Template)]
#[template(path = "index.ts.jinja", escape = "none")]
struct TypeScriptTemplate<'a> {
    pub ctx: &'a GenerationContext,
}

#[derive(Default)]
pub(crate) struct TypeScriptGenerator {}

impl Generator for TypeScriptGenerator {
    fn generate(&self, ctx: &GenerationContext, output_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(output_dir)?;

        let output_file = output_dir.join("index.ts");
        let content = generate_content(ctx)?;

        std::fs::write(output_file, content)?;

        Ok(())
    }
}

fn prettier(content: &str) -> Result<Cow<'_, str>> {
    let dir =
        tempdir().map_err(|_err| Error::TypeScriptFormat(anyhow!("couldn't create a tmp dir")))?;

    let file_path = dir.path().join("index.ts");

    let mut file = File::create(&file_path)?;

    write!(file, "{}", content)
        .map_err(|_err| Error::TypeScriptFormat(anyhow!("couldn't write content to tmp file")))?;

    let binary_path = match which("npx") {
        Err(_) => return Ok(content.into()),
        Ok(path) => path,
    };

    let mut command = Command::new(binary_path);

    command.args([
        "--yes",
        "prettier",
        "--loglevel",
        "silent",
        "--write",
        file_path.as_path().to_string_lossy().as_ref(),
    ]);

    let status = command.status().map_err(|error| {
        Error::TypeScriptFormat(anyhow!(
            "npx prettier command was not successful: {}",
            error
        ))
    })?;

    if status.success() {
        let formatted_content = read_to_string(file_path)
            .map_err(|_err| Error::TypeScriptFormat(anyhow!("couldn't read tmp file")))?;

        Ok(formatted_content.into())
    } else {
        Err(Error::TypeScriptFormat(anyhow!(
            "npx prettier command was not successful: {}",
            status
        )))
    }
}

fn generate_content(ctx: &GenerationContext) -> Result<String> {
    let content = TypeScriptTemplate { ctx }.render()?;
    let content = prettier(&content)?;

    Ok(content.into_owned())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        api_types::GenerationOptions, openapi_loader::OpenApiRefLocation, visitor::Visitor,
        OpenApiLoader,
    };

    use super::*;

    #[test]
    fn test_ts_generation() {
        let loader = OpenApiLoader::default();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/api-codegen")
            .canonicalize()
            .unwrap();
        let openapi = loader
            .load_openapi(OpenApiRefLocation::new(&root, "cars.yaml".into()))
            .unwrap();
        let ctx = GenerationContext::new(root, GenerationOptions::default());
        let ctx = Visitor::new(ctx).visit(&[openapi.clone()]).unwrap();
        let content = generate_content(&ctx).unwrap();

        insta::assert_snapshot!(content);
    }
}
