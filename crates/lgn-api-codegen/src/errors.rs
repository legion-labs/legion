use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("serde_yaml: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),
    #[error("askama: {0}")]
    Askama(#[from] askama::Error),
    #[error("rust format: {0}")]
    RustFormat(#[from] rust_format::Error),
    #[error("typescript format: {0}")]
    TypeScriptFormat(anyhow::Error),
    #[cfg(feature = "typescript-format")]
    #[error("typescript format: {0}")]
    TypeScriptParse(#[from] deno_ast::Diagnostic),
    #[error("invalid: {0}")]
    Invalid(String),
    #[error("missing operation id: {0}")]
    MissingOperationID(String),
    #[error("unsupported: {0}")]
    Unsupported(String),
}

pub type Result<T> = std::result::Result<T, Error>;
