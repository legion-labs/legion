use std::sync::Arc;
use std::{any::Any, collections::HashMap, io};

use lgn_data_model::TypeReflection;
use lgn_data_runtime::ResourceType;

use super::{OfflineResource, RefOp, ResourceHandleId, ResourceHandleUntyped, ResourceProcessor};
use crate::ResourcePathId;

/// Options which can be used to configure [`ResourceRegistry`] creation.
pub struct ResourceRegistryOptions {
    processors: HashMap<ResourceType, Box<dyn ResourceProcessor + Send + Sync>>,
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
    /// Panics if a processor for a resource of type `kind` is already
    /// registered.
    pub fn add_type_processor(
        mut self,
        ty: ResourceType,
        proc: Box<dyn ResourceProcessor + Send + Sync>,
    ) -> Self {
        let v = self.processors.insert(ty, proc).is_none();
        assert!(v);
        self
    }

    /// See `add_type_processor`
    pub fn add_type_processor_mut(
        &mut self,
        ty: ResourceType,
        proc: Box<dyn ResourceProcessor + Send + Sync>,
    ) -> &mut Self {
        let v = self.processors.insert(ty, proc).is_none();
        assert!(v);
        self
    }

    /// Same as `add_type_processor` but adds a default processor of
    /// `OfflineResource`.
    ///
    /// # Panics
    ///
    /// Panics if a processor for a resource of type `kind` is already
    /// registered.
    pub fn add_type<T: OfflineResource>(self) -> Self {
        self.add_type_processor(T::TYPE, Box::new(T::Processor::default()))
    }

    /// See `add_type`
    pub fn add_type_mut<T: OfflineResource>(&mut self) -> &mut Self {
        self.add_type_processor_mut(T::TYPE, Box::new(T::Processor::default()))
    }

    /// Creates a new registry with the options specified by `self`.
    pub fn create_registry(self) -> Arc<std::sync::Mutex<ResourceRegistry>> {
        Arc::new(std::sync::Mutex::new(ResourceRegistry::create(
            self.processors,
        )))
    }

    /// Creates a new registry with the options specified by `self`.
    pub fn create_async_registry(self) -> Arc<tokio::sync::Mutex<ResourceRegistry>> {
        Arc::new(tokio::sync::Mutex::new(ResourceRegistry::create(
            self.processors,
        )))
    }
}

/// The registry of resources currently available in memory.
///
/// While `Project` is responsible for managing on disk/source control resources
/// the `ResourceRegistry` manages resources that are in memory. Therefore it is
/// possible that some resources are in memory but not known to `Project` or are
/// part of the `Project` but are not currently loaded to memory.
pub struct ResourceRegistry {
    id_generator: ResourceHandleId,
    refcount_channel: (
        crossbeam_channel::Sender<RefOp>,
        crossbeam_channel::Receiver<RefOp>,
    ),
    ref_counts: HashMap<ResourceHandleId, isize>,
    resources: HashMap<ResourceHandleId, Option<Box<dyn Any + Send + Sync>>>,
    processors: HashMap<ResourceType, Box<dyn ResourceProcessor + Send + Sync>>,
}

impl ResourceRegistry {
    fn create(processors: HashMap<ResourceType, Box<dyn ResourceProcessor + Send + Sync>>) -> Self {
        Self {
            id_generator: 0,
            refcount_channel: crossbeam_channel::unbounded(),
            ref_counts: HashMap::new(),
            resources: HashMap::new(),
            processors,
        }
    }

    /// Create a new resource of a given type in a default state.
    ///
    /// The default state of the resource is defined by the registered
    /// `ResourceProcessor`.
    pub fn new_resource(&mut self, kind: ResourceType) -> Option<ResourceHandleUntyped> {
        if let Some(processor) = self.processors.get_mut(&kind) {
            let resource = processor.new_resource();
            Some(self.insert(resource))
        } else {
            None
        }
    }

    /// Creates an instance of a resource and deserializes content from provided
    /// reader.
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
                format!("Processor not found for '{:?}'.", kind),
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
    ) -> io::Result<(usize, Vec<ResourcePathId>)> {
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
                format!("Processor not found for '{:?}'.", kind),
            ))
        }
    }

    /// Inserts a resource into the registry and returns a handle
    /// that identifies that resource.
    fn insert(&mut self, resource: Box<dyn Any + Send + Sync>) -> ResourceHandleUntyped {
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

    /// Returns a reference to a resource behind the handle, None if the
    /// resource does not exist.
    pub fn get<'a>(&'a self, handle: &ResourceHandleUntyped) -> Option<&'a dyn Any> {
        if let Some(Some(resource)) = self.resources.get(&handle.id) {
            return Some(resource.as_ref());
        }
        None
    }

    /// Returns a mutable reference to a resource behind the handle, None if the
    /// resource does not exist.
    pub fn get_mut<'a>(&'a mut self, handle: &ResourceHandleUntyped) -> Option<&'a mut dyn Any> {
        if let Some(Some(resource)) = self.resources.get_mut(&handle.id) {
            return Some(resource.as_mut());
        }
        None
    }

    /// Returns the Properties bag of a Resource
    pub fn get_resource_reflection<'a>(
        &'a self,
        kind: ResourceType,
        handle: &ResourceHandleUntyped,
    ) -> Option<&'a dyn TypeReflection> {
        if let Some(Some(resource)) = self.resources.get(&handle.id) {
            if let Some(processor) = self.processors.get(&kind) {
                return processor.get_resource_reflection(resource.as_ref());
            }
        }
        None
    }

    /// Returns the Properties bag of a Resource
    pub fn get_resource_reflection_mut<'a>(
        &'a mut self,
        kind: ResourceType,
        handle: &ResourceHandleUntyped,
    ) -> Option<&'a mut dyn TypeReflection> {
        if let Some(Some(resource)) = self.resources.get_mut(&handle.id) {
            if let Some(processor) = self.processors.get(&kind) {
                return processor.get_resource_reflection_mut(resource.as_mut());
            }
        }
        None
    }

    /// Return the list of the Resource Type supported by the `ResourceRegisry`
    pub fn get_resource_types(&self) -> Vec<(ResourceType, &'static str)> {
        self.processors
            .iter()
            .filter_map(|(k, processor)| processor.get_resource_type_name().map(|n| (*k, n)))
            .collect()
    }

    /// Return the name of a Resource Type
    pub fn get_resource_type_name(&self, kind: ResourceType) -> Option<&'static str> {
        self.processors
            .get(&kind)
            .and_then(|processor| processor.get_resource_type_name())
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
    use std::{any::Any, io};

    use lgn_data_runtime::{resource, Asset, AssetLoader, Resource};

    use crate::{
        resource::{registry::ResourceRegistryOptions, OfflineResource, ResourceProcessor},
        ResourcePathId,
    };

    #[resource("sample")]
    struct SampleResource {
        content: String,
    }

    impl Asset for SampleResource {
        type Loader = SampleProcessor;
    }

    impl OfflineResource for SampleResource {
        type Processor = SampleProcessor;
    }

    #[derive(Default)]
    struct SampleProcessor {
        default_content: String,
    }

    impl AssetLoader for SampleProcessor {
        fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
            let mut resource = Box::new(SampleResource {
                content: String::from(""),
            });

            let mut bytes = 0usize.to_ne_bytes();
            reader.read_exact(&mut bytes)?;
            let length = usize::from_ne_bytes(bytes);

            let mut buffer = vec![0; length];
            reader.read_exact(&mut buffer)?;
            resource.content = String::from_utf8(buffer)
                .map_err(|_e| io::Error::new(io::ErrorKind::InvalidData, "Parsing error"))?;
            Ok(resource)
        }

        fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
    }

    impl ResourceProcessor for SampleProcessor {
        fn new_resource(&mut self) -> Box<dyn Any + Send + Sync> {
            Box::new(SampleResource {
                content: self.default_content.clone(),
            })
        }

        fn get_resource_type_name(&self) -> Option<&'static str> {
            Some(SampleResource::TYPENAME)
        }

        fn write_resource(
            &self,
            resource: &dyn Any,
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
        ) -> std::io::Result<Box<dyn Any + Send + Sync>> {
            self.load(reader)
        }

        fn extract_build_dependencies(&mut self, _resource: &dyn Any) -> Vec<ResourcePathId> {
            vec![]
        }
    }

    #[test]
    fn reference_count_untyped() {
        let resources = ResourceRegistryOptions::new().create_registry();
        let mut resources = resources.lock().unwrap();

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
        let resources = ResourceRegistryOptions::new().create_registry();
        let mut resources = resources.lock().unwrap();

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

        let resources = ResourceRegistryOptions::new()
            .add_type_processor(
                SampleResource::TYPE,
                Box::new(SampleProcessor {
                    default_content: String::from(default_content),
                }),
            )
            .create_registry();

        let mut resources = resources.lock().unwrap();
        assert_eq!(
            resources.get_resource_types()[0],
            (SampleResource::TYPE, SampleResource::TYPENAME)
        );

        let created_handle = resources
            .new_resource(SampleResource::TYPE)
            .expect("failed to create a resource")
            .typed::<SampleResource>();

        {
            let resource = created_handle.get(&resources).expect("resource not found");

            assert_eq!(resource.content, default_content);
        }

        let mut buffer = [0u8; 256];

        resources
            .serialize_resource(SampleResource::TYPE, &created_handle, &mut &mut buffer[..])
            .unwrap();

        let loaded_handle = resources
            .deserialize_resource(SampleResource::TYPE, &mut &buffer[..])
            .expect("Resource load")
            .typed::<SampleResource>();

        let loaded_resource = loaded_handle
            .get(&resources)
            .expect("Loaded resource not found");

        let created_resource = created_handle.get(&resources).expect("resource not found");

        assert_eq!(loaded_resource.content, created_resource.content);
    }
}
