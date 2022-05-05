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

#[derive(Debug, Template)]
#[template(path = "typescript.tpl", escape = "none")]
pub struct TypeScriptTemplate<'a, 'b> {
    entry_descriptions: &'a [EntryDescription<'b>],
}

impl<'a, 'b> TypeScriptTemplate<'a, 'b> {
    pub fn new(entry_descriptions: &'a [EntryDescription<'b>]) -> Self {
        Self { entry_descriptions }
    }
}
