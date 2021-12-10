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
    content: Vec<u8>,
}

impl Product {
    pub fn new(variant: CGenVariant, path: RelativePathBuf, content: Vec<u8>) -> Self {
        Self {
            variant,
            path,
            content,
        }
    }

    pub fn path(&self) -> &RelativePath {
        &self.path
    }

    pub fn content(&self) -> &Vec<u8> {
        &self.content
    }

    pub fn write_to_disk(&self, context: &CGenContext) -> Result<()> {
        let final_path = self.path.to_path(context.out_dir(self.variant));
        let mut dir_builder = std::fs::DirBuilder::new();
        dir_builder.recursive(true);
        dir_builder.create(final_path.parent().unwrap())?;

        trace!("Writing {}", final_path.display());
        let mut output = std::fs::File::create(&final_path)?;
        trace!("Created!");
        output.write(&self.content)?;
        trace!("Done!");

        Ok(())
    }
}
