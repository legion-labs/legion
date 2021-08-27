use std::{
    collections::hash_map::DefaultHasher,
    fmt,
    hash::{Hash, Hasher},
    num::ParseIntError,
    str::FromStr,
};

use crate::resource::ResourceId;

use legion_content_store::ContentType;
use serde::{Deserialize, Serialize};

/// Identifier of a path in a build graph.
///
/// Considering a build graph where nodes represent *resources* and edges representing *transformations* between resources
/// the `AssetPathId` uniqely identifies any resource/node in the build graph.
///
/// A tuple (`ResourceType`, `ResourceType`) identifies a transformation type between two resource types.
///
/// Each node in the graph can optionally contain a `name` property allowing to identify a specific compilation output
/// at a given node.
///
/// `AssetPathId` identifies a concrete source resource with a `ResourceId` - that also defines `ResourceType` of that resource.
/// Furthermore, it defines an ordered list of `ContentType`s this *source resource* must be transformed into during the data build process.
///
/// # Example
///
/// The following example illustrates creation of *source resource* containing geometry and
/// definition of a path representing a *derived resource* of a runtime geometry data after LOD-generation process.
///
/// ```no_run
/// # use legion_data_offline::{resource::{Project, ResourceName, ResourceRegistry, ResourceType}, asset::AssetPathId};
/// # use std::path::PathBuf;
/// # let mut resources = ResourceRegistry::default();
/// # let mut project = Project::create_new(&PathBuf::new()).unwrap();
/// # pub const SOURCE_GEOMETRY: ResourceType = ResourceType::new(b"src_geom");
/// # pub const LOD_GEOMETRY: ResourceType = ResourceType::new(b"lod_geom");
/// # pub const BINARY_GEOMETRY: ResourceType = ResourceType::new(b"bin_geom");
/// // create a resource and add it to the project
/// let resource_handle = resources.new_resource(SOURCE_GEOMETRY).unwrap();
/// let resource_id = project.add_resource(ResourceName::from("new resource"),
///                              SOURCE_GEOMETRY, &resource_handle, &mut resources).unwrap();
///
/// // create a resource path
/// let source_path = AssetPathId::from(resource_id);
/// let target = source_path.push(LOD_GEOMETRY).push(BINARY_GEOMETRY);
/// ```
#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, PartialOrd, Ord)]
pub struct AssetPathId {
    source: ResourceId,
    transforms: Vec<(ContentType, Option<String>)>,
}

impl From<ResourceId> for AssetPathId {
    fn from(id: ResourceId) -> Self {
        Self {
            source: id,
            transforms: vec![],
        }
    }
}

impl fmt::Display for AssetPathId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.source))?;
        for (kind, name) in &self.transforms {
            if let Some(name) = name {
                f.write_fmt(format_args!("|{}_{}", kind, name))?;
            } else {
                f.write_fmt(format_args!("|{}", kind))?;
            }
        }
        Ok(())
    }
}

impl FromStr for AssetPathId {
    type Err = ParseIntError;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let end = s.find('|').unwrap_or(s.len());
        let source = ResourceId::from_str(&s[0..end])?;
        s = &s[end..];

        let mut transforms = vec![];
        while !s.is_empty() {
            s = &s[1..]; // skip '|'
            let name = s.find('_').unwrap_or(s.len());
            let end = s.find('|').unwrap_or(s.len());

            let transform = if name < end {
                let err = "Z".parse::<i32>().expect_err("ParseIntError");
                let t = u32::from_str(&s[0..name])?;
                let p = String::from_str(&s[name + 1..end]).map_err(|_e| err)?;
                (ContentType::from_raw(t), Some(p))
            } else {
                let t = u32::from_str(&s[0..end])?;
                (ContentType::from_raw(t), None)
            };
            transforms.push(transform);
            s = &s[end..];
        }
        Ok(Self { source, transforms })
    }
}

impl AssetPathId {
    /// Appends a new node to the build path represented by this `AssetPathId`.
    ///
    /// The node is identified by the appended `kind`.
    /// The `AssetPathId`'s compilation output type changes to `kind`.
    pub fn push(&self, kind: ContentType) -> Self {
        let mut cloned = self.clone();
        cloned.transforms.push((kind, None));
        cloned
    }

    /// Appends a new node to the build path represented by this `AssetPathId`.
    ///
    /// The node is identified by the appended tuple of (`kind`, `name`).
    /// The `AssetPathId`'s compilation output type changes to `kind`.
    pub fn push_named(&self, kind: ContentType, name: &str) -> Self {
        let mut cloned = self.clone();
        cloned.transforms.push((kind, Some(name.to_string())));
        cloned
    }

    /// Create a new [`AssetPathId`] by changing the last node's `name` property.
    pub fn new_named(&self, name: &str) -> Self {
        assert!(!self.is_source(), "Source path cannot be named");
        let mut cloned = self.clone();
        let last_transform = cloned.transforms.last_mut().unwrap();
        last_transform.1 = Some(name.to_string());
        cloned
    }

    /// Returns `ResourceType` of the resource identified by this path.
    pub fn resource_type(&self) -> ContentType {
        if self.transforms.is_empty() {
            self.source.resource_type()
        } else {
            self.transforms[self.transforms.len() - 1].0
        }
    }

    /// Returns resource id of the build path's source resource.
    pub fn source_resource(&self) -> ResourceId {
        self.source
    }

    /// Returns an id of the build path's leaf node - the source resource.
    pub fn source_resource_path(&self) -> Self {
        Self::from(self.source)
    }

    /// Returns true if the path identifies a `source resource`.
    ///
    /// Source resource has no transformations attached to it and is backed by user-defined data.
    pub fn is_source(&self) -> bool {
        self.transforms.is_empty()
    }

    /// Returns the last transformation that must be applied to produce the resource.
    ///
    /// Returns None if self is a `source resource`.
    pub fn last_transform(&self) -> Option<(ContentType, ContentType)> {
        match self.transforms.len() {
            0 => None,
            1 => Some((self.source.resource_type(), self.transforms[0].0)),
            _ => {
                let len = self.transforms.len();
                Some((self.transforms[len - 2].0, self.transforms[len - 1].0))
            }
        }
    }

    /// Returns a `AssetPathId` that represents a direct dependency in the build graph.
    ///
    /// None if self represents a source dependency.
    pub fn direct_dependency(&self) -> Option<Self> {
        if self.is_source() {
            return None;
        }
        let mut dependency = self.clone();
        dependency.transforms.pop();
        Some(dependency)
    }

    /// Returns a hash of the `ResourceIdPath`.
    pub fn hash_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    /// Returns a hash of the name in the context of `AssetPathId`.
    pub fn hash_name(&self, name: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        name.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use crate::{
        asset::AssetPathId,
        resource::{test_resource, ResourceId},
    };

    #[test]
    fn simple_path() {
        let source = ResourceId::generate_new(test_resource::TYPE_ID);

        let path_a = AssetPathId::from(source);
        let path_b = path_a.push(test_resource::TYPE_ID);

        let name_a = format!("{}", path_a);
        assert_eq!(path_a, AssetPathId::from_str(&name_a).unwrap());

        let name_b = format!("{}", path_b);
        assert_eq!(path_b, AssetPathId::from_str(&name_b).unwrap());
    }

    #[test]
    fn named_path() {
        let source = ResourceId::generate_new(test_resource::TYPE_ID);

        let source = AssetPathId::from(source);
        let source_hello = source.push_named(test_resource::TYPE_ID, "hello");

        let hello_text = format!("{}", source_hello);
        assert_eq!(source_hello, AssetPathId::from_str(&hello_text).unwrap());
    }
}
