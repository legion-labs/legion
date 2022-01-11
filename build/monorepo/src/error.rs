use std::{
    fmt::{Debug, Display, Formatter},
    io::Write,
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// An error that can possibly inherit from a parent error.
///
/// Errors can be enriched with additional information, such as the raw output
/// of a command or a human-friendly explanation.
#[derive(thiserror::Error)]
pub struct Error {
    description: String,
    explanation: Option<String>,
    #[source]
    source: Option<anyhow::Error>,
    output: Option<String>,
    exit_code: Option<i32>,
}

impl Error {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            explanation: None,
            source: None,
            output: None,
            exit_code: None,
        }
    }

    pub fn from_source(source: impl Into<anyhow::Error>) -> Self {
        Self::new("").with_source(source)
    }

    pub fn with_source(mut self, source: impl Into<anyhow::Error>) -> Self {
        self.source = Some(source.into());

        self
    }

    pub fn with_explanation(mut self, explanation: impl Into<String>) -> Self {
        self.explanation = Some(explanation.into());

        self
    }

    pub fn with_output(mut self, output: impl Into<String>) -> Self {
        self.output = Some(output.into());

        self
    }

    pub fn with_exit_code(mut self, exit_code: Option<i32>) -> Self {
        self.exit_code = exit_code;

        self
    }

    pub fn with_context(mut self, description: impl Into<String>) -> Self {
        if self.description.is_empty() {
            self.description = description.into();

            self
        } else {
            Self::new(description).with_source(self)
        }
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn source(&self) -> Option<&anyhow::Error> {
        self.source.as_ref()
    }

    pub fn explanation(&self) -> Option<&str> {
        self.explanation.as_deref()
    }

    pub fn output(&self) -> Option<&str> {
        self.output.as_deref()
    }

    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    pub fn display(&self) {
        if atty::is(atty::Stream::Stdout) {
            let mut stderr = StandardStream::stderr(ColorChoice::Always);
            stderr
                .set_color(
                    ColorSpec::new()
                        .set_fg(Some(Color::Red))
                        .set_intense(true)
                        .set_bold(true),
                )
                .unwrap();
            write!(&mut stderr, "error").unwrap();
            stderr.reset().unwrap();
            writeln!(&mut stderr, ": {}", self.description()).unwrap();
            stderr.reset().unwrap();

            if let Some(source) = self.source() {
                stderr
                    .set_color(
                        ColorSpec::new()
                            .set_fg(Some(Color::White))
                            .set_intense(true)
                            .set_bold(true),
                    )
                    .unwrap();
                write!(&mut stderr, "Caused by").unwrap();
                stderr.reset().unwrap();
                write!(&mut stderr, ": {}", source).unwrap();
            }

            if let Some(explanation) = self.explanation() {
                stderr
                    .set_color(
                        ColorSpec::new()
                            .set_fg(Some(Color::Yellow))
                            .set_bold(true)
                            .set_intense(true),
                    )
                    .unwrap();
                write!(&mut stderr, "\n{}", explanation).unwrap();
                stderr.reset().unwrap();
            }

            if let Some(output) = self.output() {
                stderr
                    .set_color(
                        ColorSpec::new()
                            .set_fg(Some(Color::Blue))
                            .set_bold(true)
                            .set_intense(true),
                    )
                    .unwrap();
                writeln!(&mut stderr, "\nOutput follows:").unwrap();
                stderr.reset().unwrap();
                write!(&mut stderr, "{}", output).unwrap();
            }
        } else {
            eprintln!("{}", self);
        }
    }
}

pub(crate) trait ErrorContext {
    fn with_context(self, description: impl Into<String>) -> Self;
    fn with_full_context(
        self,
        description: impl Into<String>,
        explanation: impl Into<String>,
    ) -> Self;
}

impl<T> ErrorContext for crate::Result<T> {
    fn with_context(self, description: impl Into<String>) -> Self {
        self.map_err(|e| e.with_context(description))
    }

    fn with_full_context(
        self,
        description: impl Into<String>,
        explanation: impl Into<String>,
    ) -> Self {
        self.map_err(|e| e.with_context(description).with_explanation(explanation))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description)?;

        if let Some(source) = self.source.as_ref() {
            write!(f, ": {}", source)?;
        }

        if let Some(explanation) = &self.explanation {
            write!(f, "\n\n{}", explanation)?;
        }

        Ok(())
    }
}

impl Debug for Error {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        self.display();
        Ok(())
    }
}
