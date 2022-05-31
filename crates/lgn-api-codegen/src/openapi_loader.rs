use crate::api::Type;

use super::{Error, Result};

use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Display,
    path::{Component, Path, PathBuf},
    str::FromStr,
};

use http::Uri;

/// A type that is specialized in loading and parsing `OpenAPIv3` documents.
#[derive(Debug, Default)]
pub struct OpenApiLoader {
    raw_documents: RefCell<HashMap<OpenApiRefLocation, serde_json::Value>>,
}

impl OpenApiLoader {
    /// Load an `OpenAPI` document.
    ///
    /// # Errors
    ///
    /// Returns an error if the document could not be loaded.
    pub fn load_openapi(&self, ref_loc: OpenApiRefLocation) -> Result<OpenApi<'_>> {
        self.resolve_reference(ref_loc, OpenApiRef::default())
    }

    /// Resolve a reference, using the specified current context.
    ///
    /// # Errors
    ///
    /// Returns an error if the reference could not be resolved.
    pub fn resolve_reference<'s, T>(
        &'s self,
        ctx: OpenApiRefLocation,
        ref_: OpenApiRef,
    ) -> Result<OpenApiElement<'s, T>>
    where
        T: for<'de> serde::Deserialize<'de> + Send + Sync + 'static,
    {
        let cwd = std::env::current_dir()?;
        let ctx = ctx.normalized(&cwd);
        let root = match ctx.path() {
            Some(p) => p.parent().unwrap_or(&cwd),
            None => &cwd,
        };
        let ref_ = ref_
            .with_normalized_ref_location(root)
            .with_default_ref_location(ctx);

        let raw_reference = self.resolve_raw_reference(&ref_)?;
        let element: T = serde_json::from_value(raw_reference)?;

        Ok(OpenApiElement {
            loader: self,
            ref_loc: ref_.ref_location.unwrap(),
            item: OpenApiElementItem::Owned(element),
        })
    }

    pub fn get_all_files(&self) -> Vec<PathBuf> {
        self.raw_documents
            .borrow()
            .keys()
            .filter_map(OpenApiRefLocation::path)
            .cloned()
            .collect()
    }

    /// Import an element.
    ///
    /// # Errors
    ///
    /// Returns an error if the document could not be serialized or if a
    /// document already exists at the implicit location.
    #[cfg(test)]
    pub(crate) fn import<'e, E>(&'e self, element: &'e E) -> Result<OpenApiElement<'e, E>>
    where
        E: serde::Serialize,
    {
        let ref_location: OpenApiRefLocation = "api.yaml".try_into()?;
        let ref_location = ref_location.normalized(&std::env::current_dir()?);
        let mut raw_documents = self.raw_documents.borrow_mut();

        if raw_documents.contains_key(&ref_location) {
            return Err(Error::DocumentAlreadyExists(ref_location));
        }

        let raw_document = serde_json::to_value(element)?;
        raw_documents.insert(ref_location.clone(), raw_document);

        Ok(OpenApiElement {
            loader: self,
            ref_loc: ref_location,
            item: OpenApiElementItem::Ref(element),
        })
    }

    /// Import an element from its YAML representation.
    ///
    /// # Errors
    ///
    /// Returns an error if the document could not be serialized or if a
    /// document already exists at the implicit location.
    #[cfg(test)]
    pub(crate) fn import_from_yaml<E>(&self, element_yaml: &str) -> Result<OpenApiElement<'_, E>>
    where
        E: for<'de> serde::Deserialize<'de>,
    {
        let ref_location: OpenApiRefLocation = "api.yaml".try_into()?;
        let ref_location = ref_location.normalized(&std::env::current_dir()?);
        let mut raw_documents = self.raw_documents.borrow_mut();

        if raw_documents.contains_key(&ref_location) {
            return Err(Error::DocumentAlreadyExists(ref_location));
        }

        let raw_document: serde_json::Value = serde_yaml::from_str(element_yaml)?;
        raw_documents.insert(ref_location.clone(), raw_document.clone());
        let element: E = serde_json::from_value(raw_document)?;

        Ok(OpenApiElement {
            loader: self,
            ref_loc: ref_location,
            item: OpenApiElementItem::Owned(element),
        })
    }

    /// Load a document from a reference location.
    ///
    /// # Errors
    ///
    /// Returns an error if the document could not be loaded.
    fn load_raw_document(&self, ref_location: OpenApiRefLocation) -> Result<serde_json::Value> {
        let mut raw_documents = self.raw_documents.borrow_mut();

        Ok(match raw_documents.entry(ref_location) {
            std::collections::hash_map::Entry::Occupied(o) => o.into_mut(),
            std::collections::hash_map::Entry::Vacant(v) => {
                let raw_document = v.key().load()?;

                v.insert(raw_document)
            }
        }
        .clone())
    }

    /// Resolve a reference, using the specified current context.
    ///
    /// # Errors
    ///
    /// Returns an error if the reference could not be resolved.
    fn resolve_raw_reference(&self, ref_: &OpenApiRef) -> Result<serde_json::Value> {
        let document = self.load_raw_document(
            ref_.ref_location
                .as_ref()
                .expect("OpenAPI reference must have a location")
                .clone(),
        )?;
        document
            .pointer(&ref_.json_pointer)
            .cloned()
            .ok_or_else(|| Error::InvalidReference(ref_.clone()))
    }
}

/// Encapsulates a reference to an `OpenAPIv3` element, its location and the
/// associated loader.
#[derive(Debug, Clone)]
pub struct OpenApiElement<'e, E> {
    loader: &'e OpenApiLoader,
    ref_loc: OpenApiRefLocation,
    item: OpenApiElementItem<'e, E>,
}

#[derive(Debug, Clone)]
enum OpenApiElementItem<'e, E> {
    Owned(E),
    Ref(&'e E),
}

impl<'e, E> AsRef<E> for OpenApiElementItem<'e, E> {
    fn as_ref(&self) -> &E {
        match self {
            OpenApiElementItem::Owned(e) => e,
            OpenApiElementItem::Ref(e) => e,
        }
    }
}

impl<'e, E> std::ops::Deref for OpenApiElementItem<'e, E>
where
    E: std::ops::Deref,
    E::Target: Sized,
{
    type Target = E::Target;

    fn deref(&self) -> &Self::Target {
        match self {
            OpenApiElementItem::Owned(e) => &**e,
            OpenApiElementItem::Ref(e) => *e,
        }
    }
}

impl<'e, E: 'static> OpenApiElement<'e, E> {
    /// Get the location of the `OpenAPI` document.
    pub fn ref_location(&self) -> &OpenApiRefLocation {
        &self.ref_loc
    }

    /// Resolve a reference, using the current element context.
    ///
    /// # Errors
    ///
    /// Returns an error if the reference could not be resolved.
    pub fn resolve_reference<T>(&self, ref_: OpenApiRef) -> Result<OpenApiElement<'e, T>>
    where
        T: for<'de> serde::Deserialize<'de> + Send + Sync + 'static,
    {
        self.loader
            .resolve_reference(self.ref_location().clone(), ref_)
    }

    /// Resolve a reference-or, using the current element context.
    ///
    /// # Errors
    ///
    /// Returns an error if the reference could not be resolved.
    pub fn resolve_reference_or<T>(
        &'e self,
        ref_: &'e openapiv3::ReferenceOr<T>,
    ) -> Result<OpenApiElement<'e, T>>
    where
        T: for<'de> serde::Deserialize<'de> + Send + Sync + 'static,
    {
        match ref_ {
            openapiv3::ReferenceOr::Reference { reference } => {
                self.resolve_reference(reference.parse()?)
            }
            openapiv3::ReferenceOr::Item(item) => Ok(OpenApiElement {
                loader: self.loader,
                ref_loc: self.ref_loc.clone(),
                item: OpenApiElementItem::Ref(item),
            }),
        }
    }

    /// Returns a value as an `OpenApiElement` that shares the same reference
    /// location as the current instance.
    pub fn as_element_ref<T>(&self, element: &'e T) -> OpenApiElement<'_, T> {
        OpenApiElement {
            loader: self.loader,
            ref_loc: self.ref_loc.clone(),
            item: OpenApiElementItem::Ref(element),
        }
    }
}

impl<'e, E> AsRef<E> for OpenApiElement<'e, E> {
    fn as_ref(&self) -> &E {
        self.item.as_ref()
    }
}

impl<'e, E> std::ops::Deref for OpenApiElement<'e, E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        self.item.as_ref()
    }
}

/// An helper type for encapsulated `OpenAPIv3` documents.
pub type OpenApi<'e> = OpenApiElement<'e, openapiv3::OpenAPI>;

/// An `OpenAPIv3` reference.
///
/// Supports local, remote and URL references.
///
/// Use `parse()` on a string type to build.
#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct OpenApiRef {
    ref_location: Option<OpenApiRefLocation>,
    json_pointer: String,
}

impl Display for OpenApiRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.ref_location {
            Some(ref_location) => write!(f, "{}#{}", ref_location, self.json_pointer),
            None => write!(f, "{}", self.json_pointer),
        }
    }
}

impl FromStr for OpenApiRef {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.splitn(2, '#').collect::<Vec<_>>();

        if parts.len() != 2 {
            return Err(Error::Invalid(format!(
                "invalid reference: {} (did you forget the '#'?)",
                s
            )));
        }

        if parts[0].is_empty() {
            Ok(Self {
                ref_location: None,
                json_pointer: parts[1].to_string(),
            })
        } else {
            Ok(Self {
                ref_location: Some(parts[0].parse()?),
                json_pointer: parts[1].to_string(),
            })
        }
    }
}

impl From<OpenApiRefLocation> for OpenApiRef {
    fn from(ref_location: OpenApiRefLocation) -> Self {
        Self {
            ref_location: Some(ref_location),
            json_pointer: "".to_string(),
        }
    }
}

impl OpenApiRef {
    /// Sets the reference location of this instance unless it is already set.
    fn with_default_ref_location(self, ref_location: OpenApiRefLocation) -> Self {
        if self.ref_location.is_none() {
            Self {
                ref_location: Some(ref_location),
                json_pointer: self.json_pointer,
            }
        } else {
            self
        }
    }

    pub fn with_normalized_ref_location(self, root: impl AsRef<Path>) -> Self {
        match self.ref_location {
            Some(ref_location) => Self {
                ref_location: Some(ref_location.normalized(root)),
                json_pointer: self.json_pointer,
            },
            None => self,
        }
    }

    /// Returns the reference location.
    pub fn ref_location(&self) -> Option<&OpenApiRefLocation> {
        self.ref_location.as_ref()
    }

    /// Returns the JSON pointer.
    pub fn json_pointer(&self) -> &str {
        &self.json_pointer
    }

    pub fn type_name(&self) -> &str {
        self.json_pointer.rsplit('/').next().unwrap()
    }

    pub fn into_named_type(self) -> Type {
        Type::Named(self.json_pointer.rsplit('/').next().unwrap().to_string())
    }
}

/// An `OpenAPIv3` reference type.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum OpenApiRefLocation {
    Remote(PathBuf),
    Url(Uri),
}

impl Display for OpenApiRefLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenApiRefLocation::Remote(path) => write!(f, "{}", path.display()),
            OpenApiRefLocation::Url(url) => write!(f, "{}", url),
        }
    }
}

impl FromStr for OpenApiRefLocation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<Uri>() {
            Ok(uri) => uri.try_into(),
            Err(_) => Ok(PathBuf::from_str(s)
                .map_err(|e| {
                    Error::Invalid(format!("invalid OpenAPI reference location `{}`: {}", s, e))
                })?
                .into()),
        }
    }
}

impl TryFrom<&str> for OpenApiRefLocation {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl TryFrom<String> for OpenApiRefLocation {
    type Error = Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl From<PathBuf> for OpenApiRefLocation {
    fn from(p: PathBuf) -> Self {
        Self::Remote(p)
    }
}

impl TryFrom<Uri> for OpenApiRefLocation {
    type Error = Error;

    fn try_from(uri: Uri) -> Result<Self, Self::Error> {
        Ok(match uri.scheme_str() {
            Some("http" | "https") => Self::Url(uri),
            None => Self::Remote(uri.to_string().into()),

            Some(scheme) => {
                return Err(Error::Invalid(format!(
                    "unsupported scheme `{}` in reference location: {}",
                    scheme, uri
                )))
            }
        })
    }
}

impl OpenApiRefLocation {
    pub fn new_remote(path: PathBuf) -> Self {
        Self::Remote(path)
    }

    pub fn new_url(url: Uri) -> Self {
        Self::Url(url)
    }

    pub fn normalized(self, root: impl AsRef<Path>) -> Self {
        match self {
            Self::Remote(path) => Self::Remote({
                normalize_path(if path.is_absolute() {
                    path
                } else {
                    root.as_ref().join(path)
                })
            }),
            Self::Url(url) => Self::Url(url),
        }
    }

    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            Self::Remote(path) => Some(path),
            Self::Url(_) => None,
        }
    }

    /// Loads the document referenced by this reference.
    pub fn load(&self) -> Result<serde_json::Value> {
        Ok(match self {
            Self::Remote(path) => {
                let file = std::fs::File::open(path)?;
                serde_yaml::from_reader(file)?
            }
            Self::Url(uri) => {
                let resp = reqwest::blocking::get(uri.to_string())?;
                serde_yaml::from_reader(resp)?
            }
        })
    }
}

pub fn normalize_path(path: impl AsRef<Path>) -> PathBuf {
    let mut components = path.as_ref().components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().copied() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_loader_load_document() {
        let loader = OpenApiLoader::default();

        // Tests are always run from the crate's root.
        let openapi = loader
            .load_openapi("../../tests/api-codegen/cars.yaml".try_into().unwrap())
            .unwrap();

        let title: OpenApiElement<'_, String> = openapi
            .resolve_reference("#/info/title".parse().unwrap())
            .unwrap();

        assert_eq!(*title, "Test API");

        // We can even go crazy and resolve a reference directly.
        let title: OpenApiElement<'_, String> = loader
            .resolve_reference(
                "http://foo".try_into().unwrap(),
                "../../tests/api-codegen/cars.yaml#/info/title"
                    .parse()
                    .unwrap(),
            )
            .unwrap();

        assert_eq!(*title, "Test API");
    }

    #[test]
    fn test_openapi_ref_from_str() {
        assert_eq!(
            OpenApiRef::from_str("#/path/to/json").unwrap(),
            OpenApiRef {
                ref_location: None,
                json_pointer: "/path/to/json".to_string(),
            }
        );
        assert_eq!(
            OpenApiRef::from_str("some_file.json#/path/to/json").unwrap(),
            OpenApiRef {
                ref_location: Some(OpenApiRefLocation::Remote("some_file.json".into())),
                json_pointer: "/path/to/json".to_string(),
            }
        );
        assert_eq!(
            OpenApiRef::from_str("./some/relative/file.json#/path/to/json").unwrap(),
            OpenApiRef {
                ref_location: Some(OpenApiRefLocation::Remote(
                    "./some/relative/file.json".into()
                )),
                json_pointer: "/path/to/json".to_string(),
            }
        );
        assert_eq!(
            OpenApiRef::from_str("/some/absolute/file.json#/path/to/json").unwrap(),
            OpenApiRef {
                ref_location: Some(OpenApiRefLocation::Remote(
                    "/some/absolute/file.json".into()
                )),
                json_pointer: "/path/to/json".to_string(),
            }
        );
        assert_eq!(
            OpenApiRef::from_str("http://foo.bar/lol#/path/to/json").unwrap(),
            OpenApiRef {
                ref_location: Some(OpenApiRefLocation::Url(
                    "http://foo.bar/lol".parse().unwrap()
                )),
                json_pointer: "/path/to/json".to_string(),
            }
        );
        assert_eq!(
            OpenApiRef::from_str("https://foo.bar/lol#/path/to/json").unwrap(),
            OpenApiRef {
                ref_location: Some(OpenApiRefLocation::Url(
                    "https://foo.bar/lol".parse().unwrap()
                )),
                json_pointer: "/path/to/json".to_string(),
            }
        );

        OpenApiRef::from_str("/path/to/json").unwrap_err();
        OpenApiRef::from_str("bogus://foo.bar/lol#/path/to/json").unwrap_err();
    }
}
