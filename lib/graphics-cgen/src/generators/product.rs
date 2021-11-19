use anyhow::Result;
use log::trace;
use relative_path::{RelativePath, RelativePathBuf};
use std::io::Write;

use crate::run::CGenContext;

use super::CGenVariant;

#[derive(Debug)]
pub struct Product {
    variant: CGenVariant,
    path: RelativePathBuf,
    content: String,
}

impl Product {
    pub fn new(variant: CGenVariant, path: RelativePathBuf, content: String) -> Self {
        Self {
            variant,
            path,
            content,
        }
    }

    pub fn path(&self) -> &RelativePath {
        &self.path
    }

    pub fn content(&self) -> &String {
        &self.content
    }

    pub fn write_to_disk(&self, context: &CGenContext) -> Result<()> {
        let final_path = self.path.to_path(context.out_dir(self.variant));
        let mut dir_builder = std::fs::DirBuilder::new();
        dir_builder.recursive(true);
        dir_builder.create(final_path.parent().unwrap())?;

        let file_content = self.content.to_string();

        trace!("Writing {}", final_path.display());
        let mut output = std::fs::File::create(&final_path)?;
        trace!("Created!");
        output.write(&file_content.as_bytes())?;
        trace!("Done!");

        Ok(())
    }
}
