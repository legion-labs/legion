use std::{collections::HashMap, io, sync::mpsc};

use crate::{asset::AssetPathId, resource::ResourceType};

use super::{RefOp, Resource, ResourceHandleId, ResourceHandleUntyped, ResourceProcessor};

/// Options which can be used to configure [`ResourceRegistry`] creation.
pub struct ResourceRegistryOptions {
    processors: HashMap<ResourceType, Box<dyn ResourceProcessor>>,
}

impl ResourceRegistryOptions {
    /// Creates a blank set of options ready for configuration.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            processors: HashMap::new(),
        }
    }

    /// Registers a new resource processor for a given `ResourceType`.
    ///
    /// # Panics
    ///
    /// Panics if a processor for a resource of type `kind` is already registerd.
    pub fn add_type(mut self, kind: ResourceType, proc: Box<dyn ResourceProcessor>) -> Self {
        let v = self.processors.insert(kind, proc).is_none();
        assert!(v);
        self
    }

    /// Creates a new registry with the options specified by `self`.
    pub fn create_registry(self) -> ResourceRegistry {
        ResourceRegistry::create(self.processors)
    }
}

/// The registry of resources currently available in memory.
///
/// While `Project` is responsible for managing on disk/source control resources
/// the `ResourceRegisry` manages resources that are in memory. Therefore it is possible
/// that some resources are in memory but not known to `Project` or are part of the `Project`
/// but are not currently loaded to memory.
pub struct ResourceRegistry {
    id_generator: ResourceHandleId,
    refcount_channel: (mpsc::Sender<RefOp>, mpsc::Receiver<RefOp>),
    ref_counts: HashMap<ResourceHandleId, isize>,
    resources: HashMap<ResourceHandleId, Option<Box<dyn Resource>>>,
    processors: HashMap<ResourceType, Box<dyn ResourceProcessor>>,
}

impl ResourceRegistry {
    fn create(processors: HashMap<ResourceType, Box<dyn ResourceProcessor>>) -> Self {
        Self {
            id_generator: 0,
            refcount_channel: mpsc::channel(),
            ref_counts: HashMap::new(),
            resources: HashMap::new(),
            processors,
        }
    }

    /// Create a new resource of a given type in a default state.
    ///
    /// The default state of the resource is defined by the registered `ResourceProcessor`.
    pub fn new_resource(&mut self, kind: ResourceType) -> Option<ResourceHandleUntyped> {
        if let Some(processor) = self.processors.get_mut(&kind) {
            let resource = processor.new_resource();
            Some(self.insert(resource))
        } else {
            None
        }
    }

    /// Creates an instance of a resource and deserializes content from provided reader.
    pub fn deserialize_resource(
        &mut self,
        kind: ResourceType,
        reader: &mut dyn io::Read,
    ) -> io::Result<ResourceHandleUntyped> {
        if let Some(processor) = self.processors.get_mut(&kind) {
            let resource = processor.read_resource(reader)?;
            Ok(self.insert(resource))
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Processor not found.",
            ))
        }
    }

    fn create_handle(&mut self) -> ResourceHandleUntyped {
        self.id_generator += 1;
        let new_id = self.id_generator;
        // insert data
        self.ref_counts.insert(new_id, 1);
        ResourceHandleUntyped::create(new_id, self.refcount_channel.0.clone())
    }

    /// Serializes the content of the resource into the writer.
    pub fn serialize_resource(
        &mut self,
        kind: ResourceType,
        handle: impl AsRef<ResourceHandleUntyped>,
        writer: &mut dyn io::Write,
    ) -> io::Result<(usize, Vec<AssetPathId>)> {
        if let Some(processor) = self.processors.get_mut(&kind) {
            let resource = self
                .resources
                .get(&handle.as_ref().id)
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Resource not found."))?
                .as_ref()
                .unwrap()
                .as_ref();

            let build_deps = processor.extract_build_dependencies(&*resource);
            let written = processor.write_resource(&*resource, writer)?;
            Ok((written, build_deps))
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Processor not found.",
            ))
        }
    }

    /// Inserts a resource into the registry and returns a handle
    /// that identifies that resource.
    fn insert(&mut self, resource: Box<dyn Resource>) -> ResourceHandleUntyped {
        let handle = self.create_handle();
        self.resources.insert(handle.id, Some(resource));
        handle
    }

    /// Frees all the resources that have no handles pointing to them.
    pub fn collect_garbage(&mut self) {
        while let Ok(op) = self.refcount_channel.1.try_recv() {
            match op {
                RefOp::AddRef(id) => {
                    let count = self.ref_counts.get_mut(&id).unwrap();
                    *count += 1;
                }
                RefOp::RemoveRef(id) => {
                    let count = self.ref_counts.get_mut(&id).unwrap();
                    *count -= 1;
                    if *count == 0 {
                        self.remove_handle(id);
                    }
                }
            }
        }
    }

    fn remove_handle(&mut self, handle_id: ResourceHandleId) {
        // remove data
        if let Some(rc) = self.ref_counts.remove(&handle_id) {
            self.resources.remove(&handle_id);
            assert_eq!(rc, 0);
        }
    }

    /// Returns a reference to a resource behind the handle, None if the resource does not exist.
    pub fn get<'a>(&'a self, handle: &ResourceHandleUntyped) -> Option<&'a dyn Resource> {
        if let Some(Some(resource)) = self.resources.get(&handle.id) {
            return Some(resource.as_ref());
        }
        None
    }

    /// Returns a mutable reference to a resource behind the handle, None if the resource does not exist.
    pub fn get_mut<'a>(
        &'a mut self,
        handle: &ResourceHandleUntyped,
    ) -> Option<&'a mut dyn Resource> {
        if let Some(Some(resource)) = self.resources.get_mut(&handle.id) {
            return Some(resource.as_mut());
        }
        None
    }

    /// Returns the number of loaded resources.
    pub fn len(&self) -> usize {
        self.ref_counts.len()
    }

    /// Checks if this `ResourceRegistry` is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use crate::{
        asset::AssetPathId,
        resource::{registry::ResourceRegistryOptions, ResourceProcessor, ResourceType},
    };

    use super::Resource;

    struct SampleResource {
        content: String,
    }

    impl Resource for SampleResource {}

    struct SampleProcessor {
        default_content: String,
    }

    impl ResourceProcessor for SampleProcessor {
        fn new_resource(&mut self) -> Box<dyn Resource> {
            Box::new(SampleResource {
                content: self.default_content.clone(),
            })
        }

        fn write_resource(
            &mut self,
            resource: &dyn Resource,
            writer: &mut dyn std::io::Write,
        ) -> std::io::Result<usize> {
            let resource = resource.downcast_ref::<SampleResource>().unwrap();

            let length = resource.content.len();
            writer.write_all(&length.to_ne_bytes())?;
            writer.write_all(resource.content.as_bytes())?;
            Ok(length.to_ne_bytes().len() + resource.content.as_bytes().len())
        }

        fn read_resource(
            &mut self,
            reader: &mut dyn std::io::Read,
        ) -> std::io::Result<Box<dyn Resource>> {
            let mut resource = self.new_resource();
            let sample_resource = resource.downcast_mut::<SampleResource>().unwrap();

            let mut bytes = 0usize.to_ne_bytes();
            reader.read_exact(&mut bytes)?;
            let length = usize::from_ne_bytes(bytes);

            let mut buffer = vec![0; length];
            reader.read_exact(&mut buffer)?;
            sample_resource.content = String::from_utf8(buffer)
                .map_err(|_e| io::Error::new(io::ErrorKind::InvalidData, "Parsing error"))?;
            Ok(resource)
        }

        fn extract_build_dependencies(&mut self, _resource: &dyn Resource) -> Vec<AssetPathId> {
            vec![]
        }
    }

    const RESOURCE_SAMPLE: ResourceType = ResourceType::new(b"sample");

    #[test]
    fn reference_count_untyped() {
        let mut resources = ResourceRegistryOptions::new().create_registry();

        {
            let handle = resources
                .insert(Box::new(SampleResource {
                    content: String::from("test content"),
                }))
                .typed::<SampleResource>();
            assert!(handle.get(&resources).is_some());
            assert_eq!(resources.len(), 1);

            {
                let alias = handle.clone();
                assert!(alias.get(&resources).is_some());
            }
            resources.collect_garbage();
            assert_eq!(resources.len(), 1);

            assert!(handle.get(&resources).is_some());
        }

        resources.collect_garbage();
        assert_eq!(resources.len(), 0);

        {
            let handle = resources
                .insert(Box::new(SampleResource {
                    content: String::from("more test content"),
                }))
                .typed::<SampleResource>();
            assert!(handle.get(&resources).is_some());
            assert_eq!(resources.len(), 1);
        }
    }

    #[test]
    fn reference_count_typed() {
        let mut resources = ResourceRegistryOptions::new().create_registry();

        {
            let handle = resources
                .insert(Box::new(SampleResource {
                    content: String::from("test content"),
                }))
                .typed::<SampleResource>();
            assert!(handle.get(&resources).is_some());
            assert_eq!(resources.len(), 1);

            {
                let alias = handle.clone();
                assert!(alias.get(&resources).is_some());
            }
            resources.collect_garbage();
            assert_eq!(resources.len(), 1);

            assert!(handle.get(&resources).is_some());
        }

        resources.collect_garbage();
        assert_eq!(resources.len(), 0);

        {
            let handle = resources
                .insert(Box::new(SampleResource {
                    content: String::from("more test content"),
                }))
                .typed::<SampleResource>();
            assert!(handle.get(&resources).is_some());
            assert_eq!(resources.len(), 1);
        }
    }

    #[test]
    fn create_save_load() {
        let default_content = "default content";

        let mut resources = ResourceRegistryOptions::new()
            .add_type(
                RESOURCE_SAMPLE,
                Box::new(SampleProcessor {
                    default_content: String::from(default_content),
                }),
            )
            .create_registry();

        let created_handle = resources
            .new_resource(RESOURCE_SAMPLE)
            .expect("failed to create a resource")
            .typed::<SampleResource>();

        {
            let resource = created_handle.get(&resources).expect("resource not found");

            assert_eq!(resource.content, default_content);
        }

        let mut buffer = [0u8; 256];

        resources
            .serialize_resource(RESOURCE_SAMPLE, &created_handle, &mut &mut buffer[..])
            .unwrap();

        let loaded_handle = resources
            .deserialize_resource(RESOURCE_SAMPLE, &mut &buffer[..])
            .expect("Resource load")
            .typed::<SampleResource>();

        let loaded_resource = loaded_handle
            .get(&resources)
            .expect("Loaded resource not found");

        let created_resource = created_handle.get(&resources).expect("resource not found");

        assert_eq!(loaded_resource.content, created_resource.content);
    }
}
