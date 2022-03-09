use std::{fmt, hash::Hash, str::FromStr};

use lgn_data_model::{implement_primitive_type_def, ReflectionError};
use lgn_data_runtime::{ResourceId, ResourceType, ResourceTypeAndId};
use serde::{Deserialize, Serialize};

/// Resource transformation identifier.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct Transform {
    from: ResourceType,
    to: ResourceType,
}

impl Transform {
    /// Creates a new resource transform.
    pub const fn new(from: ResourceType, to: ResourceType) -> Self {
        Self { from, to }
    }
}

impl fmt::Display for Transform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}-{}", self.from, self.to))
    }
}

impl FromStr for Transform {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split(|c| c == '-');
        let from = iter
            .next()
            .ok_or_else(|| "Z".parse::<u64>().expect_err("ParseIntError"))?;
        let to = iter
            .next()
            .ok_or_else(|| "Z".parse::<u64>().expect_err("ParseIntError"))?;

        Ok(Self {
            from: ResourceType::from_str(from)?,
            to: ResourceType::from_str(to)?,
        })
    }
}

/// Identifier of a path in a build graph.
///
/// Considering a build graph where nodes represent *resources* and edges
/// representing *transformations* between resources the `ResourcePathId`
/// uniquely identifies any resource/node in the build graph.
///
/// A tuple (`ResourceType`, `ResourceType`) identifies a transformation type
/// between two resource types.
///
/// Each node in the graph can optionally contain a `name` property allowing to
/// identify a specific compilation output at a given node.
///
/// `ResourcePathId` identifies a concrete source resource with a `ResourceId` -
/// that also defines `ResourceType` of that resource. Furthermore, it defines
/// an ordered list of `ContentType`s this *source resource* must be transformed
/// into during the data build process.
///
/// # Example
///
/// The following example illustrates creation of *source resource* containing
/// geometry and definition of a path representing a *derived resource* of a
/// runtime geometry data after LOD-generation process.
///
/// ```no_run
/// use lgn_data_offline::resource::{Project, ResourcePathName, ResourceRegistryOptions};
/// use lgn_data_runtime::ResourceType;
/// use lgn_data_offline::ResourcePathId;
/// use std::path::PathBuf;
/// # tokio_test::block_on(async {
/// let resources = ResourceRegistryOptions::new().create_registry();
/// let mut resources = resources.lock().unwrap();
/// let mut project = Project::create_with_remote_mock(&PathBuf::new()).await.unwrap();
/// pub const SOURCE_GEOMETRY: &'static str = "src_geom";
/// pub const LOD_GEOMETRY: ResourceType = ResourceType::new(b"lod_geom");
/// pub const BINARY_GEOMETRY: ResourceType = ResourceType::new(b"bin_geom");
/// // create a resource and add it to the project
/// let source_geometry_type = ResourceType::new(SOURCE_GEOMETRY.as_bytes());
/// let resource_handle = resources.new_resource(source_geometry_type).unwrap();
/// let resource_id = project
///     .add_resource(
///         ResourcePathName::new("new resource"),
///         SOURCE_GEOMETRY,
///         source_geometry_type,
///         &resource_handle,
///         &mut resources,
///     ).await
///     .unwrap();
/// // create a resource path
/// let source_path = ResourcePathId::from(resource_id);
/// let _target = source_path.push(LOD_GEOMETRY).push(BINARY_GEOMETRY);
/// # })
/// ```
#[derive(Hash, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct ResourcePathId {
    source: ResourceTypeAndId,
    transforms: Vec<(ResourceType, Option<String>)>,
}

implement_primitive_type_def!(
    ResourcePathId,
    Err(ReflectionError::UnsupportedDefault("ResourcePathId"))
);

impl From<ResourceTypeAndId> for ResourcePathId {
    fn from(type_id: ResourceTypeAndId) -> Self {
        Self {
            source: type_id,
            transforms: vec![],
        }
    }
}

impl fmt::Display for ResourcePathId {
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

impl fmt::Debug for ResourcePathId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self.source))?;
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

impl FromStr for ResourcePathId {
    type Err = Box<dyn std::error::Error>;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let end = s.find('|').unwrap_or(s.len());
        let source = s[0..end].parse::<ResourceTypeAndId>().unwrap();
        s = &s[end..];

        let mut transforms = vec![];
        while !s.is_empty() {
            s = &s[1..]; // skip '|'
            let name = s.find('_').unwrap_or(s.len());
            let end = s.find('|').unwrap_or(s.len());

            let transform = if name < end {
                let t = ResourceType::from_str(&s[0..name])?;
                let p = String::from_str(&s[name + 1..end])
                    .map_err(|_e| "Z".parse::<i32>().expect_err("ParseIntError"))?;
                (t, Some(p))
            } else {
                let t = ResourceType::from_str(&s[0..end])?;
                (t, None)
            };
            transforms.push(transform);
            s = &s[end..];
        }
        Ok(Self { source, transforms })
    }
}

impl Serialize for ResourcePathId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ResourcePathId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str = String::deserialize(deserializer)?;
        Self::from_str(&str).map_err(|_e| serde::de::Error::custom("Parse Error"))
    }
}

impl ResourcePathId {
    /// Appends a new node to the build path represented by this
    /// `ResourcePathId`.
    ///
    /// The node is identified by the appended `kind`.
    /// The `ResourcePathId`'s compilation output type changes to `kind`.
    pub fn push(&self, kind: impl Into<ResourceType>) -> Self {
        let mut cloned = self.clone();
        cloned.transforms.push((kind.into(), None));
        cloned
    }

    /// Appends a new node to the build path represented by this
    /// `ResourcePathId`.
    ///
    /// The node is identified by the appended tuple of (`kind`, `name`).
    /// The `ResourcePathId`'s compilation output type changes to `kind`.
    pub fn push_named(&self, kind: impl Into<ResourceType>, name: &str) -> Self {
        let mut cloned = self.clone();
        cloned
            .transforms
            .push((kind.into(), Some(name.to_string())));
        cloned
    }

    /// Create a new [`ResourcePathId`] by changing the last node's `name`
    /// property.
    pub fn new_named(&self, name: &str) -> Self {
        assert!(!self.is_source(), "Source path cannot be named");
        let mut cloned = self.clone();
        let last_transform = cloned.transforms.last_mut().unwrap();
        last_transform.1 = Some(name.to_string());
        cloned
    }

    /// Creates a new id without the `name` part.
    pub fn to_unnamed(&self) -> Self {
        let mut cloned = self.clone();
        if let Some((_, name)) = cloned.transforms.last_mut() {
            *name = None;
        }
        cloned
    }

    /// Returns true if last transformation contains `name` part, false
    /// otherwise.
    pub fn is_named(&self) -> bool {
        if let Some((_, name)) = self.transforms.last() {
            !name.is_none()
        } else {
            false
        }
    }

    /// Returns `name` part of the id.
    pub fn name(&self) -> Option<&str> {
        if let Some((_, Some(name))) = self.transforms.last() {
            Some(name)
        } else {
            None
        }
    }

    /// Returns `ResourceType` of the resource identified by this path.
    pub fn content_type(&self) -> ResourceType {
        if self.transforms.is_empty() {
            self.source.kind
        } else {
            self.transforms.last().unwrap().0
        }
    }

    /// Returns resource id of the build path's source resource.
    pub fn source_resource(&self) -> ResourceTypeAndId {
        self.source
    }

    /// Returns an id of the build path's leaf node - the source resource.
    pub fn source_resource_path(&self) -> Self {
        Self::from(self.source)
    }

    /// Returns true if the path identifies a `source resource`.
    ///
    /// Source resource has no transformations attached to it and is backed by
    /// user-defined data.
    pub fn is_source(&self) -> bool {
        self.transforms.is_empty()
    }

    /// Returns the last transformation that must be applied to produce the
    /// resource.
    ///
    /// Returns None if self is a `source resource`.
    pub fn last_transform(&self) -> Option<Transform> {
        match self.transforms.len() {
            0 => None,
            1 => Some(Transform::new(self.source.kind, self.transforms[0].0)),
            _ => {
                let len = self.transforms.len();
                Some(Transform::new(
                    self.transforms[len - 2].0,
                    self.transforms[len - 1].0,
                ))
            }
        }
    }

    /// Returns a `ResourcePathId` that represents a direct dependency in the
    /// build graph.
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

    /// Returns an identifier representing the path.
    pub fn resource_id(&self) -> ResourceTypeAndId {
        if self.is_source() {
            self.source
        } else {
            ResourceTypeAndId {
                kind: self.content_type(),
                id: ResourceId::from_obj(&self),
            }
        }
    }

    /// Produces an iterator over transformations contained within the resource
    /// path.
    ///
    /// # Example
    ///
    /// ```
    /// # use lgn_data_runtime::{ResourceType, ResourceId, ResourceTypeAndId};
    /// # use lgn_data_offline::{ResourcePathId};
    /// # const FOO_TYPE: ResourceType = ResourceType::new(b"foo");
    /// # const BAR_TYPE: ResourceType = ResourceType::new(b"bar");
    /// let source = ResourceTypeAndId { kind: FOO_TYPE, id: ResourceId::new() };
    /// let path = ResourcePathId::from(source).push(BAR_TYPE).push_named(FOO_TYPE, "parameter");
    ///
    /// let mut transforms = path.transforms();
    /// assert_eq!(transforms.next(), Some((FOO_TYPE, BAR_TYPE, None)));
    /// assert_eq!(transforms.next(), Some((BAR_TYPE, FOO_TYPE, Some(&"parameter".to_string()))));
    /// assert_eq!(transforms.next(), None);
    /// ```
    pub fn transforms(&self) -> Transforms<'_> {
        Transforms {
            path_id: self,
            target_index: 0,
        }
    }
}

/// An iterator over the transformations of a [`ResourcePathId`].
///
/// This struct is created by the [`transforms`] method on [`ResourcePathId`].
///
/// [`transforms`]: ResourcePathId::transforms
pub struct Transforms<'a> {
    path_id: &'a ResourcePathId,
    target_index: usize,
}

impl<'a> Iterator for Transforms<'a> {
    type Item = (ResourceType, ResourceType, Option<&'a String>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.target_index < self.path_id.transforms.len() {
            let source = if self.target_index == 0 {
                self.path_id.source.kind
            } else {
                self.path_id.transforms[self.target_index - 1].0
            };
            let (target, name) = &self.path_id.transforms[self.target_index];
            let out = Some((source, *target, name.as_ref()));
            self.target_index += 1;
            out
        } else {
            None
        }
    }
}

use lgn_data_model::{
    collector::{ItemInfo, PropertyCollector},
    TypeDefinition,
};
use std::collections::HashSet;
/// Extract the build dependencies using reflection
pub fn extract_resource_dependencies(
    object: &dyn lgn_data_model::TypeReflection,
) -> Option<Vec<ResourcePathId>> {
    struct ExtractResourcePathId {
        output: HashSet<ResourcePathId>,
    }

    impl PropertyCollector for ExtractResourcePathId {
        type Item = Option<Self>;
        fn new_item(item_info: &ItemInfo<'_>) -> Result<Self::Item, ReflectionError> {
            if let TypeDefinition::Primitive(primitive_descriptor) = item_info.type_def {
                if primitive_descriptor.base_descriptor.type_name == "ResourcePathId" {
                    if let Some(file_descriptor) = item_info.field_descriptor {
                        if file_descriptor
                            .attributes
                            .as_ref()
                            .map_or(false, |attr| attr.contains_key("ignore_deps"))
                        {
                            return Ok(None);
                        }
                    }

                    let mut output = Vec::new();
                    let mut json = serde_json::Serializer::new(&mut output);
                    let mut serializer = <dyn erased_serde::Serializer>::erase(&mut json);
                    unsafe {
                        (primitive_descriptor.base_descriptor.dynamic_serialize)(
                            item_info.base,
                            &mut serializer,
                        )?;
                    }

                    let path = String::from_utf8(output)?;

                    if let Ok(res_id) =
                        ResourcePathId::from_str(path.trim_start_matches('"').trim_end_matches('"'))
                    {
                        let mut result = Self {
                            output: HashSet::with_capacity(1),
                        };
                        result.output.insert(res_id);
                        return Ok(Some(result));
                    }
                }
            }
            Ok(None)
        }
        fn add_child(parent: &mut Self::Item, child: Self::Item) {
            if let Some(child) = child {
                parent
                    .get_or_insert(Self {
                        output: HashSet::new(),
                    })
                    .output
                    .extend(child.output);
            }
        }
    }

    if let Ok(Some(total)) =
        lgn_data_model::collector::collect_properties::<ExtractResourcePathId>(object)
    {
        let mut result = total.output.into_iter().collect::<Vec<_>>();
        result.sort();
        return Some(result);
    }
    None
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use lgn_data_runtime::{Resource, ResourceId, ResourceType, ResourceTypeAndId};

    use crate::{resource::test_resource, ResourcePathId, Transform};

    #[test]
    fn simple_path() {
        let source = ResourceTypeAndId {
            kind: test_resource::TestResource::TYPE,
            id: ResourceId::new(),
        };

        let path_a = ResourcePathId::from(source);
        let path_b = path_a.push(test_resource::TestResource::TYPE);

        let name_a = path_a.to_string();
        assert_eq!(path_a, ResourcePathId::from_str(&name_a).unwrap());

        let name_b = path_b.to_string();
        assert_eq!(path_b, ResourcePathId::from_str(&name_b).unwrap());
    }

    #[test]
    fn transform() {
        let source = Transform::new(
            test_resource::TestResource::TYPE,
            test_resource::TestResource::TYPE,
        );

        let text = source.to_string();
        assert!(text.len() > 1);
        assert!(text.contains('-'));

        let parsed = Transform::from_str(&text).expect("parsed Transform");
        assert_eq!(source, parsed);
    }

    #[test]
    fn named_path() {
        let source = ResourceTypeAndId {
            kind: test_resource::TestResource::TYPE,
            id: ResourceId::new(),
        };

        let source = ResourcePathId::from(source);
        let source_hello = source.push_named(test_resource::TestResource::TYPE, "hello");

        let hello_text = source_hello.to_string();
        assert_eq!(source_hello, ResourcePathId::from_str(&hello_text).unwrap());
    }

    #[test]
    fn transform_iter() {
        let foo_type = ResourceType::new(b"foo");
        let bar_type = ResourceType::new(b"bar");
        let source = ResourceTypeAndId {
            kind: foo_type,
            id: ResourceId::new(),
        };

        let source_only = ResourcePathId::from(source);
        assert_eq!(source_only.transforms().next(), None);

        let path = ResourcePathId::from(source)
            .push(bar_type)
            .push_named(foo_type, "test_name");

        let mut transform_iter = path.transforms();
        assert_eq!(transform_iter.next(), Some((foo_type, bar_type, None)));
        assert_eq!(
            transform_iter.next(),
            Some((bar_type, foo_type, Some(&"test_name".to_string())))
        );
        assert_eq!(transform_iter.next(), None);
        assert_eq!(transform_iter.next(), None);
    }
}
