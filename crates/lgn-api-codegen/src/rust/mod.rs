mod filters;

use crate::{
    api::{Api, MediaType, Type},
    Generator, Result,
};
use askama::Template;
use rust_format::{Formatter, RustFmt};
use std::{ffi::OsStr, path::Path};

#[derive(askama::Template)]
#[template(path = "lib.rs.jinja", escape = "none")]
struct RustTemplate<'a> {
    pub api: &'a Api,
}

#[derive(Default)]
pub(crate) struct RustGenerator {}

impl Generator for RustGenerator {
    fn generate(&self, api: &Api, openapi_file: &Path, output_dir: &Path) -> Result<()> {
        let content = generate(api)?;

        let output_file = output_dir.join(
            openapi_file
                .to_path_buf()
                .with_extension("rs")
                .file_name()
                .unwrap_or_else(|| OsStr::new("openapi.rs")),
        );

        std::fs::create_dir_all(output_dir)?;
        std::fs::write(output_file, content)?;

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
    use crate::OpenApiLoader;

    use super::*;

    #[test]
    fn test_rust_generation() {
        let loader = OpenApiLoader::default();
        let openapi = loader
            .load_openapi("../../tests/api-codegen/cars.yaml".try_into().unwrap())
            .unwrap();
        let api = openapi.try_into().unwrap();
        let content = generate(&api).unwrap();

        insta::assert_snapshot!(content);
    }
}
