mod filters;

use crate::{
    api_types::{GenerationContext, MediaType, Type},
    Generator, Result,
};
use askama::Template;
use rust_format::{Formatter, RustFmt};
use std::path::Path;

#[derive(askama::Template)]
#[template(path = "lib.rs.jinja", escape = "none")]
struct RustTemplate<'a> {
    pub ctx: &'a GenerationContext,
}

#[derive(Default)]
pub(crate) struct RustGenerator {}

impl Generator for RustGenerator {
    fn generate(&self, ctx: &GenerationContext, output_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(output_dir)?;

        let output_file = output_dir.join("api.rs");
        let content = generate_content(ctx)?;

        std::fs::write(output_file, content)?;

        Ok(())
    }
}

fn generate_content(ctx: &GenerationContext) -> Result<String> {
    let content = RustTemplate { ctx }.render()?;
    let content = RustFmt::default().format_str(&content)?;

    Ok(content)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{openapi_loader::OpenApiRefLocation, visitor::Visitor, OpenApiLoader};

    use super::*;

    #[test]
    fn test_rust_generation() {
        let loader = OpenApiLoader::default();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/api-codegen")
            .canonicalize()
            .unwrap();
        let openapi = loader
            .load_openapi(OpenApiRefLocation::new(&root, "cars.yaml".into()))
            .unwrap();
        let ctx = Visitor::new(root).visit(&[openapi.clone()]).unwrap();
        let content = generate_content(&ctx).unwrap();

        insta::assert_snapshot!(content);
    }
}
