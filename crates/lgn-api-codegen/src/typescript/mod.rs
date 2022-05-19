use std::path::Path;
#[cfg(feature = "typescript-format")]
use std::sync::Arc;

#[cfg(feature = "typescript-format")]
use anyhow::anyhow;
use askama::Template;
#[cfg(feature = "typescript-format")]
use deno_ast::{parse_module, MediaType as DenoAstMediaType, ParseParams, SourceTextInfo};
#[cfg(feature = "typescript-format")]
use dprint_plugin_typescript::{configuration::ConfigurationBuilder, format_parsed_source};

#[cfg(feature = "typescript-format")]
use crate::Error;
use crate::{
    api::{Api, MediaType, Model},
    Generator, Result,
};

mod filters;

#[derive(askama::Template)]
#[template(path = "index.ts.jinja", escape = "none")]
struct TypeScriptTemplate<'a> {
    pub api: &'a Api,
}

#[derive(Default)]
pub(crate) struct TypeScriptGenerator {}

impl Generator for TypeScriptGenerator {
    fn generate(&self, api: &Api, output_dir: &Path) -> Result<()> {
        let content = generate(api)?;

        #[cfg(feature = "typescript-format")]
        let content = format(content)?;

        std::fs::create_dir_all(output_dir)?;
        std::fs::write(output_dir.join("index.ts"), content)?;

        Ok(())
    }
}

fn generate(api: &Api) -> Result<String> {
    let content = TypeScriptTemplate { api }.render()?;

    Ok(content)
}

#[cfg(feature = "typescript-format")]
fn format(content: String) -> Result<String> {
    let source = SourceTextInfo::new(Arc::new(content));

    let parsed_source = parse_module(ParseParams {
        specifier: "".to_string(),
        media_type: DenoAstMediaType::TypeScript,
        source,
        capture_tokens: true,
        maybe_syntax: None,
        scope_analysis: false,
    })?;

    let configuration = ConfigurationBuilder::new().build();

    let content =
        format_parsed_source(&parsed_source, &configuration).map_err(Error::TypeScriptFormat)?;

    let content = content
        .ok_or_else(|| Error::TypeScriptFormat(anyhow!("Couldn't format the typescript source")))?;

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ts_generation() {
        let data = include_str!("./fixtures/openapi.yaml");
        let api = Api::try_from(&serde_yaml::from_str(data).unwrap()).unwrap();
        let content = generate(&api).unwrap();

        insta::assert_snapshot!(content);
    }
}
