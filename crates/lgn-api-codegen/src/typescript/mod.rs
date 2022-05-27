use std::{ffi::OsStr, path::Path};

use askama::Template;

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
    fn generate(&self, api: &Api, openapi_file: &Path, output_dir: &Path) -> Result<()> {
        let content = generate(api)?;

        let output_file = output_dir.join(
            openapi_file
                .to_path_buf()
                .with_extension("ts")
                .file_name()
                .unwrap_or_else(|| OsStr::new("index.ts")),
        );

        std::fs::create_dir_all(output_dir)?;
        std::fs::write(output_file, content)?;

        Ok(())
    }
}

fn generate(api: &Api) -> Result<String> {
    let content = TypeScriptTemplate { api }.render()?;

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ts_generation() {
        let data = include_str!("../fixtures/openapi.yaml");
        let api = Api::try_from(&serde_yaml::from_str(data).unwrap()).unwrap();
        let content = generate(&api).unwrap();

        insta::assert_snapshot!(content);
    }
}
