use std::collections::HashMap;

use crate::{visitor, Error, Result};
use convert_case::{Case, Casing};
use indexmap::IndexMap;

#[derive(Debug, PartialEq)]
pub struct Api {
    pub title: String,
    pub description: Option<String>,
    pub version: String,
    pub models: Vec<Model>,
    pub paths: HashMap<Path, Vec<Route>>,
}

impl TryFrom<&openapiv3::OpenAPI> for Api {
    type Error = Error;

    fn try_from(openapi: &openapiv3::OpenAPI) -> Result<Self> {
        visitor::visit(openapi)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int32,
    Int64,
    String,
    Boolean,
    Float32,
    Float64,
    DateTime,
    Date,
    Bytes,
    Binary,
    Array(Box<Type>),
    HashSet(Box<Type>),
    Struct(String),
}

#[derive(Debug, PartialEq)]
pub enum Model {
    Enum(Enum),
    Struct(Struct),
}

impl Model {
    pub fn name(&self) -> &str {
        match self {
            Model::Enum(enum_) => &enum_.name,
            Model::Struct(struct_) => &struct_.name,
        }
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct Enum {
    pub name: String,
    pub description: Option<String>,
    pub variants: Vec<String>,
}

#[derive(Debug, PartialEq, Default)]
pub struct Struct {
    pub name: String,
    pub description: Option<String>,
    pub fields: Vec<Field>,
}

#[derive(Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub description: Option<String>,
    pub type_: Type,
    pub required: bool,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Path(pub String);

impl From<&str> for Path {
    fn from(path: &str) -> Self {
        Self(path.to_string())
    }
}

#[derive(Debug, PartialEq)]
pub struct Route {
    pub name: String,
    pub method: Method,
    pub summary: Option<String>,
    pub request_body: Option<RequestBody>,
    pub parameters: Parameters,
    pub responses: IndexMap<StatusCode, Response>,
}

#[derive(Debug, PartialEq, Default)]
pub struct Parameters {
    pub path: Vec<Parameter>,
    pub query: Vec<Parameter>,
    pub header: Vec<Parameter>,
    pub cookie: Vec<Parameter>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct StatusCode(http::StatusCode);

impl From<http::StatusCode> for StatusCode {
    fn from(status_code: http::StatusCode) -> Self {
        Self(status_code)
    }
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            self.0
                .canonical_reason()
                .unwrap_or(&format!("Status{}", self.0.as_u16()))
                .to_case(Case::Pascal)
                .as_str(),
        )
    }
}

impl StatusCode {
    pub(crate) fn as_u16(&self) -> u16 {
        self.0.as_u16()
    }
}

#[derive(Debug, PartialEq)]
pub struct Response {
    pub description: String,
    pub content: Option<Content>,
}

#[derive(Debug, PartialEq)]
pub struct Content {
    pub media_type: MediaType,
    pub type_: Type,
}

#[derive(Debug, PartialEq)]
pub enum MediaType {
    Bytes,
    Json,
}

impl TryFrom<&str> for MediaType {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self> {
        match s {
            "application/octet-stream" => Ok(Self::Bytes),
            "application/json" => Ok(Self::Json),
            _ => Err(Error::Invalid(format!("media type: {}", s))),
        }
    }
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Bytes => "application/octet-stream",
            Self::Json => "application/json",
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum Method {
    Get,
    Post,
    Delete,
    Put,
    Patch,
    Head,
    Options,
    Trace,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Delete => "DELETE",
            Method::Put => "PUT",
            Method::Patch => "PATCH",
            Method::Head => "HEAD",
            Method::Options => "OPTIONS",
            Method::Trace => "TRACE",
        })
    }
}

impl std::str::FromStr for Method {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "DELETE" => Ok(Self::Delete),
            "PUT" => Ok(Self::Put),
            "PATCH" => Ok(Self::Patch),
            "HEAD" => Ok(Self::Head),
            "OPTIONS" => Ok(Self::Options),
            "TRACE" => Ok(Self::Trace),
            _ => Err(Error::Invalid(format!("method: {}", s))),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct RequestBody {
    pub description: Option<String>,
    pub required: bool,
    pub content: Content,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub description: Option<String>,
    pub type_: Type,
    pub required: bool,
}
