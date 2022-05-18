mod filters;

use crate::{
    api::{Api, MediaType, Model},
    Generator, Result,
};
use askama::Template;
use rust_format::{Formatter, RustFmt};
use std::path::Path;

#[derive(askama::Template)]
#[template(path = "lib.rs.jinja", escape = "none")]
struct RustTemplate<'a> {
    pub api: &'a Api,
}

#[derive(Default)]
pub(crate) struct RustGenerator {}

impl Generator for RustGenerator {
    fn generate(&self, api: &Api, output_dir: &Path) -> Result<()> {
        let content = generate(api)?;

        std::fs::create_dir_all(output_dir)?;
        std::fs::write(output_dir.join("lib.rs"), content)?;

        Ok(())
    }
}

fn generate(api: &Api) -> Result<String> {
    let content = RustTemplate { api }.render()?;
    let content = RustFmt::default().format_str(&content)?;

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_generation() {
        let data = include_str!("./fixtures/openapi.yaml");
        let api = Api::try_from(&serde_yaml::from_str(data).unwrap()).unwrap();
        let content = generate(&api).unwrap();

        insta::assert_snapshot!(content);
    }
}
