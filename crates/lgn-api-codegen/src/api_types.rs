use std::{
    collections::BTreeMap, fmt::Display, iter::Chain, path::PathBuf, slice::Iter, str::FromStr,
};

use crate::{
    openapi_loader::{JsonPointer, OpenApiRef, OpenApiRefLocation},
    Error, Result,
};

/// Apis is the top level container for all APIs and shared models.
#[derive(Debug, PartialEq)]
pub struct GenerationContext {
    pub root: PathBuf,
    pub location_contexts: BTreeMap<OpenApiRefLocation, LocationContext>,
}

impl GenerationContext {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            location_contexts: BTreeMap::new(),
        }
    }

    pub fn ref_loc_to_module_path(&self, ref_loc: &OpenApiRefLocation) -> Result<ModulePath> {
        let file_path = ref_loc.path();

        let file_path =
            file_path
                .strip_prefix(&self.root)
                .map_err(|_err| Error::DocumentOutOfRoot {
                    document_path: file_path.clone(),
                    root: self.root.clone(),
                })?;

        Ok(ModulePath(
            file_path
                .with_extension("")
                .display()
                .to_string()
                .split('/')
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
        ))
    }

    /// Returns the location context as modules relative to the root.
    pub fn as_modules(&self) -> Result<BTreeMap<ModulePath, &LocationContext>> {
        self.location_contexts
            .iter()
            .map(|(ref_loc, api_ctx)| {
                let module_path = self.ref_loc_to_module_path(ref_loc)?;

                Ok((module_path, api_ctx))
            })
            .collect()
    }

    pub fn get_model(&self, ref_: &OpenApiRef) -> Result<&Model> {
        let location_context = self
            .location_contexts
            .get(ref_.ref_location())
            .ok_or_else(|| Error::BrokenReference(ref_.clone()))?;

        location_context
            .models
            .get(ref_.json_pointer())
            .ok_or_else(|| Error::BrokenReference(ref_.clone()))
    }
}

/// Represents a module path.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModulePath(pub Vec<String>);

impl Display for ModulePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join("/"))
    }
}

impl FromStr for ModulePath {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.split('/').map(ToString::to_string).collect()))
    }
}

impl ModulePath {
    pub fn join(&self, module: impl Into<String>) -> ModulePath {
        let mut parts = self.0.clone();
        parts.push(module.into());

        Self(parts)
    }

    pub fn parent(&self) -> ModulePath {
        Self(self.0[..self.0.len() - 1].to_vec())
    }

    pub fn relative_to(&self, other: &ModulePath) -> ModulePath {
        for (i, part) in self.0.iter().enumerate() {
            if i >= other.0.len() {
                // The other is a prefix of us: just strip the beginnning.
                return Self(self.0[i..].to_vec());
            }

            if part != &other.0[i] {
                let mut parts = other.0[i..]
                    .iter()
                    .map(|_| "..".to_string())
                    .collect::<Vec<_>>();

                parts.extend(self.0[i..].iter().cloned());

                return Self(parts);
            }
        }

        Self(
            other.0[self.0.len()..]
                .iter()
                .map(|_| "..".to_string())
                .collect(),
        )
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct LocationContext {
    pub api: Option<Api>,
    pub models: BTreeMap<JsonPointer, Model>,
}

/// API is the resolved type that is fed to templates and contains helper
/// methods to ease their writing.
#[derive(Debug, Default, PartialEq)]
pub struct Api {
    pub title: String,
    pub description: Option<String>,
    pub version: String,
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
    Named(OpenApiRef),
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
pub enum ModelOrigin {
    /// The model is defined as a schema.
    Schemas,
    /// The model is defined as a property of an object.
    ObjectProperty { object_pointer: JsonPointer },
    /// The model is auto-generated from a request body type.
    RequestBody { operation_name: String },
    /// The model is auto-generated from a response body type.
    ResponseBody {
        operation_name: String,
        status_code: StatusCode,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Model {
    pub ref_: OpenApiRef,
    pub description: Option<String>,
    pub origin: ModelOrigin,
    pub type_: Type,
}

impl Model {
    pub fn to_named_type(&self) -> Type {
        Type::Named(self.ref_.clone())
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

    pub fn has_no_responses_content(&self) -> bool {
        for resp in self.responses.values() {
            if resp.content.is_some() {
                return false;
            }
        }

        true
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl FromStr for MediaType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "application/octet-stream" => Ok(Self::Bytes),
            "application/json" => Ok(Self::Json),
            _ => Err(Error::UnsupportedMediaType(s.to_string())),
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
            _ => Err(Error::UnsupportedMethod(s.to_string())),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_path() {
        assert_eq!(
            ModulePath(vec!["foo".to_string(), "bar".to_string()]),
            "foo/bar".parse().unwrap()
        );
    }

    #[test]
    fn test_module_path_relative_to() {
        let current: ModulePath = "foo/bar/baz".parse().unwrap();
        let other: ModulePath = "foo/bar".parse().unwrap();

        assert_eq!(other.relative_to(&current), "..".parse().unwrap());
        assert_eq!(current.relative_to(&other), "baz".parse().unwrap());
    }
}
