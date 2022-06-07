use std::{ffi::OsStr, path::Path};

use crate::{
    api::{Api, MediaType, Model, Type},
    Generator, Result,
};
use askama::Template;
use lazy_static::__Deref;

mod filters;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_py_generation() {
        let data = include_str!("../../../../tests/api-codegen/cars.yaml");
        let api = Api::try_from(&serde_yaml::from_str(data).unwrap()).unwrap();
        let content = generate(&api).unwrap();

        insta::assert_snapshot!(content);
    }
}
