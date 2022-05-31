use thiserror::Error;

use crate::openapi_loader::{OpenApiRef, OpenApiRefLocation};

#[derive(Error, Debug)]
pub enum Error {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("serde_json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("serde_yaml: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),
    #[error("reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("askama: {0}")]
    Askama(#[from] askama::Error),
    #[error("uri: {0}")]
    Uri(#[from] http::uri::InvalidUri),
    #[error("rust format: {0}")]
    RustFormat(#[from] rust_format::Error),
    #[error("typescript format: {0}")]
    TypeScriptFormat(anyhow::Error),
    #[error("invalid: {0}")]
    Invalid(String),
    #[error("invalid reference: {0}")]
    InvalidReference(OpenApiRef),
    #[error("missing operation id: {0}")]
    MissingOperationID(String),
    #[error("document already exists at location: {0}")]
    DocumentAlreadyExists(OpenApiRefLocation),
    #[error("unsupported: {0}")]
    Unsupported(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
