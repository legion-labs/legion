use std::{
    borrow::Cow,
    fs::{create_dir_all, File},
    io,
    path::Path,
};

use askama::Template;
use fluent_syntax::ast::{Entry, Expression, InlineExpression, PatternElement};

use crate::error::{Error, Result};

#[derive(Debug)]
pub struct EntryDescription<'a> {
    /// The id of the message
    id: &'a str,
    /// The attributes as described [here](https://projectfluent.org/fluent/guide/attributes.html)
    attributes: Vec<&'a str>,
    /// The variables as described [here](https://projectfluent.org/fluent/guide/variables.html)
    variables: Vec<&'a str>,
}

impl<'a> TryFrom<&'a Entry<&'a str>> for EntryDescription<'a> {
    type Error = Error;

    fn try_from(entry: &'a Entry<&'a str>) -> Result<Self> {
        if let Entry::Message(message) = entry {
            let id = message.id.name;

            let attributes = message
                .attributes
                .iter()
                .map(|attribute| attribute.id.name)
                .collect::<Vec<_>>();

            let variables = message.value.as_ref().map_or(Vec::new(), |value| {
                value
                    .elements
                    .iter()
                    .filter_map(|element| {
                        if let PatternElement::Placeable { expression } = element {
                            if let Expression::Inline(InlineExpression::VariableReference { id })
                            | Expression::Select {
                                selector: InlineExpression::VariableReference { id },
                                ..
                            } = expression
                            {
                                return Some(id.name);
                            }
                        };

                        None
                    })
                    .collect::<Vec<_>>()
            });

            Ok(Self {
                id,
                attributes,
                variables,
            })
        } else {
            Err(Error::EntryNotMessage)
        }
    }
}

/// Extension of the [`Template`] trait
pub trait RenderableTemplate<'a>: Template {
    fn file_name(&'a self) -> Cow<'a, str>;

    /// Pretty much the `render_into` mehtod provided by the `Template` trait
    /// but it accepts an `io::Write` instead of an `fmt::Write`
    fn render_into_write(&'a self, writer: &mut (impl io::Write + ?Sized)) -> Result<()> {
        let content = self.render()?;

        writer.write_all(content.as_bytes())?;

        Ok(())
    }

    /// Write the [`String`] content into a file, if a file under the provided directory.
    ///
    /// # Errors
    ///
    /// If the provided directory already exists but is _not_ a directory, then an error occurs
    fn render_to_dir<P: AsRef<Path>>(&'a self, out_dir: P) -> Result<()> {
        let out_dir = out_dir.as_ref();

        if !out_dir.exists() {
            create_dir_all(&out_dir)?;
        }

        if !out_dir.is_dir() {
            return Err(Error::OutDirNotDir);
        }

        let mut file = File::create(out_dir.join(self.file_name().as_ref()))?;

        self.render_into_write(&mut file)?;

        Ok(())
    }
}

#[derive(Debug, Template)]
#[template(path = "fluent.d.ts.jinja", escape = "none")]
pub struct TypeScriptTemplate<'a, 'b> {
    entry_descriptions: &'a [EntryDescription<'b>],
    file_name: String,
}

impl<'a, 'b, 'c> RenderableTemplate<'c> for TypeScriptTemplate<'a, 'b> {
    fn file_name(&'c self) -> Cow<'c, str> {
        (&self.file_name).into()
    }
}

impl<'a, 'b> TypeScriptTemplate<'a, 'b> {
    // TODO: Accept custom file name coming from the outside
    pub fn new(entry_descriptions: &'a [EntryDescription<'b>]) -> Self {
        Self {
            entry_descriptions,
            file_name: "fluent.d.ts".into(),
        }
    }
}
