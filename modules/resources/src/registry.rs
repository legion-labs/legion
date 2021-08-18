use core::fmt;
use std::{
    any::Any,
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
};

use slotmap::{SecondaryMap, SlotMap};

use crate::{ResourcePathId, ResourceType};

slotmap::new_key_type!(
    struct ResourceHandleId;
);

/// Types implementing `Resource` represent editor data.
pub trait Resource: Any {
    /// Cast to &dyn Any type.
    fn as_any(&self) -> &dyn Any;

    /// Cast to &mut dyn Any type.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// The `ResourceProcessor` trait allows to process an offline resource.
pub trait ResourceProcessor {
    /// Interface returning a resource in a default state. Useful when creating a new resource.
    fn new_resource(&mut self) -> Box<dyn Resource>;

    /// Interface returning a list of resources that `resource` depends on for building.
    fn extract_build_dependencies(&mut self, resource: &dyn Resource) -> Vec<ResourcePathId>;

    /// Interface defining serialization behavior of the resource.
    fn write_resource(
        &mut self,
        resource: &dyn Resource,
        writer: &mut dyn io::Write,
    ) -> io::Result<usize>;

    /// Interface defining deserialization behavior of the resource.
    fn read_resource(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Resource>>;
}

/// Reference counted handle to a resource.
pub struct ResourceHandle {
    handle_id: ResourceHandleId,
    shared: Arc<Mutex<ResourceRefCounter>>,
}

impl ResourceHandle {
    fn new(shared: Arc<Mutex<ResourceRefCounter>>) -> Self {
        let id = shared.lock().unwrap().ref_counts.insert(1);
        Self {
            handle_id: id,
            shared,
        }
    }

    /// Returns a reference to the resource behind the handle if one exists.
    pub fn get<'a, T: Resource>(&'_ self, registry: &'a ResourceRegistry) -> Option<&'a T> {
        let resource = registry.get(self)?;
        resource.as_any().downcast_ref::<T>()
    }

    /// Returns a reference to the resource behind the handle if one exists.
    pub fn get_mut<'a, T: Resource>(
        &'_ self,
        registry: &'a mut ResourceRegistry,
    ) -> Option<&'a mut T> {
        let resource = registry.get_mut(self)?;
        resource.as_any_mut().downcast_mut::<T>()
    }
}

impl fmt::Debug for ResourceHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceHandle")
            .field("handle_id", &self.handle_id)
            .finish()
    }
}

impl PartialEq for ResourceHandle {
    fn eq(&self, other: &Self) -> bool {
        self.handle_id == other.handle_id
    }
}

impl Eq for ResourceHandle {}

impl Clone for ResourceHandle {
    fn clone(&self) -> Self {
        self.shared.lock().unwrap().increase(self.handle_id);
        Self {
            shared: self.shared.clone(),
            handle_id: self.handle_id,
        }
    }
}

impl Drop for ResourceHandle {
    fn drop(&mut self) {
        self.shared.lock().unwrap().decrease(self.handle_id);
    }
}

#[derive(Default)]
struct ResourceRefCounter {
    ref_counts: SlotMap<ResourceHandleId, isize>,
    orphans: Vec<ResourceHandleId>,
}

impl ResourceRefCounter {
    fn increase(&mut self, handle_id: ResourceHandleId) {
        // SAFETY: This method is only called by an object containing a reference therefore
        // it is safe to assume the reference count exists.
        let ref_count = unsafe { self.ref_counts.get_unchecked_mut(handle_id) };
        *ref_count += 1;
    }

    fn decrease(&mut self, handle_id: ResourceHandleId) {
        // SAFETY: This method is only called by an object containing a reference therefore
        // it is safe to assume the reference count exists. If the reference count reaches
        // zero the slotmap entry is removed.
        let ref_count = unsafe { self.ref_counts.get_unchecked_mut(handle_id) };
        let count = *ref_count - 1;
        *ref_count = count;
        if count == 0 {
            self.ref_counts.remove(handle_id);
            self.orphans.push(handle_id);
        }
    }
}

/// The registry of resources currently available in memory.
///
/// While `Project` is responsible for managing on disk/source control resources
/// the `ResourceRegisry` manages resources that are in memory. Therefore it is possible
/// that some resources are in memory but not known to `Project` or are part of the `Project`
/// but are not currently loaded to memory.
#[derive(Default)]
pub struct ResourceRegistry {
    ref_counts: Arc<Mutex<ResourceRefCounter>>,
    resources: SecondaryMap<ResourceHandleId, Option<Box<dyn Resource>>>,
    processors: HashMap<ResourceType, Box<dyn ResourceProcessor>>,
}

impl ResourceRegistry {
    /// Register a processor for a resource type.
    ///
    /// A [`ResourceProcessor`] will allow to serialize/deserialize and extract load dependecies from a resource.
    pub fn register_type(&mut self, kind: ResourceType, proc: Box<dyn ResourceProcessor>) {
        self.processors.insert(kind, proc);
    }

    /// Create a new resource of a given type in a default state.
    ///
    /// The default state of the resource is defined by the registered `ResourceProcessor`.
    pub fn new_resource(&mut self, kind: ResourceType) -> Option<ResourceHandle> {
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
    ) -> io::Result<ResourceHandle> {
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

    /// Serializes the content of the resource into the writer.
    pub fn serialize_resource(
        &mut self,
        kind: ResourceType,
        handle: &ResourceHandle,
        writer: &mut dyn io::Write,
    ) -> io::Result<(usize, Vec<ResourcePathId>)> {
        if let Some(processor) = self.processors.get_mut(&kind) {
            if self
                .ref_counts
                .lock()
                .unwrap()
                .ref_counts
                .contains_key(handle.handle_id)
            {
                let resource = self
                    .resources
                    .get(handle.handle_id)
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
                    "Resource not found.",
                ))
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Processor not found.",
            ))
        }
    }

    /// Inserts a resource into the registry and returns a handle
    /// that identifies that resource.
    pub fn insert(&mut self, resource: Box<dyn Resource>) -> ResourceHandle {
        let handle = ResourceHandle::new(self.ref_counts.clone());
        self.resources.insert(handle.handle_id, Some(resource));
        handle
    }

    /// Frees all the resources that have no handles pointing to them.
    pub fn collect_garbage(&mut self) {
        for orphan in std::mem::take(&mut self.ref_counts.lock().unwrap().orphans) {
            self.resources[orphan] = None;
        }
    }

    /// Returns a reference to a resource behind the handle, None if the resource does not exist.
    pub fn get<'a>(&'a self, handle: &ResourceHandle) -> Option<&'a dyn Resource> {
        if self
            .ref_counts
            .lock()
            .unwrap()
            .ref_counts
            .contains_key(handle.handle_id)
        {
            Some(
                self.resources
                    .get(handle.handle_id)?
                    .as_ref()
                    .unwrap()
                    .as_ref(),
            )
        } else {
            None
        }
    }

    /// Returns a mutable reference to a resource behind the handle, None if the resource does not exist.
    pub fn get_mut<'a>(&'a mut self, handle: &ResourceHandle) -> Option<&'a mut dyn Resource> {
        if self
            .ref_counts
            .lock()
            .unwrap()
            .ref_counts
            .contains_key(handle.handle_id)
        {
            Some(
                self.resources
                    .get_mut(handle.handle_id)?
                    .as_mut()
                    .unwrap()
                    .as_mut(),
            )
        } else {
            None
        }
    }

    /// Returns the number of loaded resources.
    pub fn len(&self) -> usize {
        self.ref_counts.lock().unwrap().ref_counts.len()
    }

    /// Checks if this `ResourceRegistry` is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io,
        sync::{Arc, Mutex},
    };

    use crate::{ResourcePathId, ResourceProcessor, ResourceType};

    use super::{Resource, ResourceHandle, ResourceRefCounter, ResourceRegistry};

    #[test]
    fn ref_count() {
        let counter = Arc::new(Mutex::new(ResourceRefCounter::default()));

        let ref_a = ResourceHandle::new(counter.clone());
        let ref_b = ResourceHandle::new(counter);

        assert_ne!(ref_a, ref_b);
    }

    struct SampleResource {
        content: String,
    }

    impl Resource for SampleResource {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

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
            let resource = resource.as_any().downcast_ref::<SampleResource>().unwrap();

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
            let sample_resource = resource
                .as_any_mut()
                .downcast_mut::<SampleResource>()
                .unwrap();

            let mut bytes = 0usize.to_ne_bytes();
            reader.read_exact(&mut bytes)?;
            let length = usize::from_ne_bytes(bytes);

            let mut buffer = vec![0; length];
            reader.read_exact(&mut buffer)?;
            sample_resource.content = String::from_utf8(buffer)
                .map_err(|_e| io::Error::new(io::ErrorKind::InvalidData, "Parsing error"))?;
            Ok(resource)
        }

        fn extract_build_dependencies(&mut self, _resource: &dyn Resource) -> Vec<ResourcePathId> {
            vec![]
        }
    }

    const RESOURCE_SAMPLE: ResourceType = ResourceType::new(b"sample");

    #[test]
    fn reference_count() {
        let mut resources = ResourceRegistry::default();

        {
            let handle = resources.insert(Box::new(SampleResource {
                content: String::from("test content"),
            }));
            assert!(handle.get::<SampleResource>(&resources).is_some());
            assert_eq!(resources.len(), 1);

            {
                let alias = handle.clone();
                assert!(alias.get::<SampleResource>(&resources).is_some());
            }
            resources.collect_garbage();
            assert_eq!(resources.len(), 1);

            assert!(handle.get::<SampleResource>(&resources).is_some());
        }

        resources.collect_garbage();
        assert_eq!(resources.len(), 0);

        {
            let handle = resources.insert(Box::new(SampleResource {
                content: String::from("more test content"),
            }));
            assert!(handle.get::<SampleResource>(&resources).is_some());
            assert_eq!(resources.len(), 1);
        }
    }

    #[test]
    fn create_save_load() {
        let default_content = "default content";

        let mut resources = {
            let mut reg = ResourceRegistry::default();

            reg.register_type(
                RESOURCE_SAMPLE,
                Box::new(SampleProcessor {
                    default_content: String::from(default_content),
                }),
            );
            reg
        };

        let created_handle = resources
            .new_resource(RESOURCE_SAMPLE)
            .expect("failed to create a resource");

        {
            let resource = created_handle
                .get::<SampleResource>(&resources)
                .expect("resource not found");

            assert_eq!(resource.content, default_content);
        }

        let mut buffer = [0u8; 256];

        resources
            .serialize_resource(RESOURCE_SAMPLE, &created_handle, &mut &mut buffer[..])
            .unwrap();

        let loaded_handle = resources
            .deserialize_resource(RESOURCE_SAMPLE, &mut &buffer[..])
            .expect("Resource load");

        let loaded_resource = loaded_handle
            .get::<SampleResource>(&resources)
            .expect("Loaded resource not found");

        let created_resource = created_handle
            .get::<SampleResource>(&resources)
            .expect("resource not found");

        assert_eq!(loaded_resource.content, created_resource.content);
    }
}
