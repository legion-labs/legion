use anyhow::Result;
use relative_path::{RelativePath, RelativePathBuf};
use std::{io::Write};

use super::{CGenVariant, GeneratorContext};

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

    pub fn write_to_disk(&self, context: &GeneratorContext<'_>) -> Result<()> {
        let final_path = self.path.to_path(context.get_base_folder(self.variant));
        let mut dir_builder = std::fs::DirBuilder::new();
        dir_builder.recursive(true);
        dir_builder.create(final_path.parent().unwrap())?;

        let file_content = self.content.to_string();

        let mut output = std::fs::File::create(&final_path)?;
        output.write(&file_content.as_bytes())?;

        Ok(())
    }
}
