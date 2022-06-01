use std::{collections::BTreeMap, iter::Chain, slice::Iter};

use crate::{Error, OpenAPIPath, Result};

/// API is the resolved type that is fed to templates and contains helper
/// methods to ease their writing.
#[derive(Debug, Default, PartialEq)]
pub struct Api {
    pub title: String,
    pub description: Option<String>,
    pub version: String,
    pub models: BTreeMap<String, Model>,
    pub paths: BTreeMap<Path, Vec<Route>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
    Array(Box<Self>),
    HashSet(Box<Self>),
    Named(String),
    Enum { variants: Vec<String> },
    Struct { fields: BTreeMap<String, Field> },
    OneOf { types: Vec<Self> },
}

impl Type {
    pub fn requires_model(&self) -> bool {
        matches!(
            self,
            Type::Enum { .. } | Type::Struct { .. } | Type::OneOf { .. }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Model {
    pub name: String,
    pub description: Option<String>,
    pub type_: Type,
}

impl Model {
    pub fn to_named_type(&self) -> Type {
        Type::Named(self.name.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Field {
    pub name: String,
    pub description: Option<String>,
    pub type_: Type,
    pub required: bool,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Path(pub String);

impl From<&OpenAPIPath> for Path {
    fn from(path: &OpenAPIPath) -> Self {
        Self(path.to_string())
    }
}

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
    pub responses: BTreeMap<StatusCode, Response>,
}

impl Route {
    pub fn has_empty_request(&self) -> bool {
        self.request_body.is_none() && self.parameters.is_empty()
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct Parameters {
    pub path: Vec<Parameter>,
    pub query: Vec<Parameter>,
    pub header: Vec<Parameter>,
    pub cookie: Vec<Parameter>,
}

impl Parameters {
    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
            && self.query.is_empty()
            && self.header.is_empty()
            && self.cookie.is_empty()
    }
}

impl<'a> IntoIterator for &'a Parameters {
    type Item = &'a Parameter;
    type IntoIter = Chain<
        Chain<Chain<Iter<'a, Parameter>, Iter<'a, Parameter>>, Iter<'a, Parameter>>,
        Iter<'a, Parameter>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.path
            .iter()
            .chain(self.query.iter())
            .chain(self.header.iter())
            .chain(self.cookie.iter())
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StatusCode(http::StatusCode);

impl From<http::StatusCode> for StatusCode {
    fn from(status_code: http::StatusCode) -> Self {
        Self(status_code)
    }
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.0.as_u16()))
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
    pub headers: BTreeMap<String, Header>,
}

#[derive(Debug, PartialEq)]
pub struct Content {
    pub media_type: MediaType,
    pub type_: Type,
}

#[derive(Debug, PartialEq)]
pub struct Header {
    pub description: Option<String>,
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
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Delete => "DELETE",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
            Self::Trace => "TRACE",
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
