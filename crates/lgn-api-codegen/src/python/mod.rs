use std::{ffi::OsStr, path::Path};

use crate::{api::Api, Generator, Result};
use askama::Template;

#[derive(askama::Template)]
#[template(path = "__init__.py.jinja", escape = "none")]
struct PythonTemplate<'a> {
    pub api: &'a Api,
}

#[derive(Default)]
pub(crate) struct PythonGenerator {}

impl Generator for PythonGenerator {
    fn generate(&self, api: &Api, openapi_file: &Path, output_dir: &Path) -> Result<()> {
        let content = generate(api)?;

        let output_file = output_dir.join(
            openapi_file
                .to_path_buf()
                .with_extension("py")
                .file_name()
                .unwrap_or_else(|| OsStr::new("openapi.py")),
        );

        std::fs::create_dir_all(output_dir)?;
        std::fs::write(output_file, content)?;

        Ok(())
    }
}

fn generate(api: &Api) -> Result<String> {
    let content = PythonTemplate { api }.render()?;

    Ok(content)
}
