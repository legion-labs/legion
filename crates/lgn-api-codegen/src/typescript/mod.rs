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
    Generator, Result, TypeScriptOptions,
};

mod filters;

#[derive(askama::Template)]
#[template(path = "index.ts.jinja", escape = "none")]
struct TypeScriptIndexTemplate<'a> {
    pub ctx: &'a GenerationContext,
}

#[derive(askama::Template)]
#[template(path = "package.json.jinja", escape = "none")]
struct TypeScriptPackageTemplate;

#[derive(Default)]
pub(crate) struct TypeScriptGenerator {
    // prettier_config_path: Option<PathBuf>,
    // with_package_json: bool,
    // skip_format: bool,
}

impl TypeScriptGenerator {
    fn generate_index_content(
        &self,
        ctx: &GenerationContext,
        options: &TypeScriptOptions,
    ) -> Result<String> {
        let mut content = TypeScriptIndexTemplate { ctx }.render()?;

        if !options.skip_format {
            content = self.format(options, &content, "index.ts")?.into_owned();
        }

        Ok(content)
    }

    fn generate_package_content(&self, options: &TypeScriptOptions) -> Result<String> {
        let mut content = TypeScriptPackageTemplate.render()?;

        if !options.skip_format {
            content = self.format(options, &content, "package.json")?.into_owned();
        }

        Ok(content)
    }

    #[allow(clippy::unused_self)]
    fn format<'a, 'b>(
        &self,
        options: &TypeScriptOptions,
        content: &'a str,
        temp_file_name: &'b str,
    ) -> Result<Cow<'a, str>> {
        let prettier_config_path = options
            .prettier_config_path
            .clone()
            .map(|path| path.to_string_lossy().into_owned());

        let dir = tempdir()
            .map_err(|_err| Error::TypeScriptFormat(anyhow!("couldn't create a tmp dir")))?;

        let file_path = dir.path().join(temp_file_name);

        let mut file = File::create(&file_path)?;

        write!(file, "{}", content).map_err(|_err| {
            Error::TypeScriptFormat(anyhow!("couldn't write content to tmp file"))
        })?;

        let binary_path = match which("npx") {
            Err(_) => return Ok(content.into()),
            Ok(path) => path,
        };

        let mut command = Command::new(binary_path);

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

        command.args(&args);

        let status = command.status().map_err(|error| {
            Error::TypeScriptFormat(anyhow!(
                "npx prettier command was not successful: {}",
                error
            ))
        })?;

        if status.success() {
            let formatted_content = read_to_string(&file_path)
                .map_err(|_err| Error::TypeScriptFormat(anyhow!("couldn't read tmp file")))?;

            Ok(formatted_content.into())
        } else {
            Err(Error::TypeScriptFormat(anyhow!(
                "npx prettier command was not successful: {}",
                status
            )))
        }
    }
}

impl Generator for TypeScriptGenerator {
    fn generate(&self, ctx: &GenerationContext, output_dir: &Path) -> Result<()> {
        let default_options = TypeScriptOptions::default();

        let options = ctx
            .language
            .type_script_options()
            .unwrap_or(&default_options);

        std::fs::create_dir_all(output_dir)?;

        let output_file = output_dir.join("index.ts");
        let content = self.generate_index_content(ctx, options)?;

        std::fs::write(output_file, content)?;

        if options.with_package_json {
            let output_file = output_dir.join("package.json");
            let content = self.generate_package_content(options)?;

            std::fs::write(output_file, content)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        api_types::Language, openapi_loader::OpenApiRefLocation, visitor::Visitor, OpenApiLoader,
    };

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
        let ctx = GenerationContext::new(root, Language::TypeScript(TypeScriptOptions::default()));
        let ctx = Visitor::new(ctx).visit(&[openapi.clone()]).unwrap();
        let content = TypeScriptGenerator::default()
            .generate_index_content(&ctx, &TypeScriptOptions::default())
            .unwrap();

        insta::assert_snapshot!(content);
    }

    #[test]
    fn test_ts_package_generation() {
        let content = TypeScriptGenerator::default()
            .generate_package_content(&TypeScriptOptions::default())
            .unwrap();

        insta::assert_snapshot!(content);
    }
}
