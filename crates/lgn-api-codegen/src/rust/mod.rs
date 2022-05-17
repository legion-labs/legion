mod filters;

use crate::{
    api::{Api, ContentType, Model},
    Generator, Result,
};
use askama::Template;
use rust_format::{Formatter, RustFmt};
use std::path::Path;

#[derive(askama::Template)]
#[template(path = "lib.rs.jinja", escape = "none")]
pub struct RustTemplate<'a> {
    pub api: &'a Api,
}

#[derive(Default)]
pub struct RustGenerator {}

impl Generator for RustGenerator {
    fn generate(&self, api: &Api, output_dir: &Path) -> Result<()> {
        let content = RustTemplate { api }.render()?;
        let content = RustFmt::default().format_str(&content)?;

        std::fs::create_dir_all(output_dir)?;
        std::fs::write(output_dir.join("lib.rs"), content)?;

        Ok(())
    }
}
