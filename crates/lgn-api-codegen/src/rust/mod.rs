mod filters;

use super::{
    api::{Api, Model},
    Generator,
};
use crate::Result;
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

impl<'a> Generator<'a> for RustGenerator {
    fn generate(&self, api: &'a Api, output_dir: &Path) -> Result<()> {
        let content = RustTemplate { api }.render()?;
        let content = RustFmt::default().format_str(&content)?;

        std::fs::create_dir_all(output_dir)?;
        std::fs::write(output_dir.join("lib.rs"), content)?;

        Ok(())
    }
}
