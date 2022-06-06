use std::{ffi::OsStr, path::Path};

use askama::Template;

use crate::{
    api_types::{Api, GenerationContext, MediaType},
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
    fn generate(&self, ctx: &GenerationContext, output_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(output_dir)?;

        for (ref_loc, api_ctx) in &ctx.location_contexts {
            if let Some(api) = &api_ctx.api {
                let content = generate_api(api)?;

                let output_file = output_dir.join(
                    ref_loc
                        .path()
                        .with_extension("ts")
                        .file_name()
                        .unwrap_or_else(|| OsStr::new("index.ts")),
                );
                std::fs::write(output_file, content)?;
            }

            //TODO: Generate models.
        }

        Ok(())
    }
}

fn generate_api(api: &Api) -> Result<String> {
    let content = TypeScriptTemplate { api }.render()?;

    Ok(content)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{openapi_loader::OpenApiRefLocation, visitor::Visitor, OpenApiLoader};

    use super::*;

    #[ignore]
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
        let ctx = Visitor::new(root).visit(&[openapi.clone()]).unwrap();
        let content = generate_api(
            ctx.location_contexts
                .get(openapi.ref_().ref_location())
                .unwrap()
                .api
                .as_ref()
                .unwrap(),
        )
        .unwrap();

        insta::assert_snapshot!(content);
    }
}
