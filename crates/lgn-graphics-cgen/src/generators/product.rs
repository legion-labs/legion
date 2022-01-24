use std::io::Write;

use anyhow::Result;
use relative_path::RelativePath;

use super::CGenVariant;
use crate::run::CGenContext;

#[derive(Debug)]
pub struct Product {
    variant: CGenVariant,
    path: String,
    content: Vec<u8>,
}

impl Product {
    pub fn new(variant: CGenVariant, path: String, content: Vec<u8>) -> Self {
        Self {
            variant,
            path,
            content,
        }
    }

    pub fn path(&self) -> &String {
        &self.path
    }

    pub fn content(&self) -> &Vec<u8> {
        &self.content
    }

    /// Write product content to disk.
    ///
    /// # Errors
    /// Todo.
    pub fn write_to_disk(&self, context: &CGenContext) -> Result<()> {
        // create output folder if needed
        let final_path = RelativePath::new(&self.path).to_path(context.out_dir(self.variant));
        let mut dir_builder = std::fs::DirBuilder::new();
        dir_builder.recursive(true);
        dir_builder.create(final_path.parent().unwrap())?;

        // create file
        let mut output = std::fs::File::create(&final_path)?;

        // write file footer
        writeln!(output, "// This is generated file. Do not edit manually")?;
        writeln!(output)?;

        // write file content
        output.write_all(&self.content)?;
        output.flush()?;

        Ok(())
    }
}
