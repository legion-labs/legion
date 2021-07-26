use core::fmt;
use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use slotmap::{SecondaryMap, SlotMap};

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
        registry.get(self)
    }

    /// Returns a reference to the resource behind the handle if one exists.
    pub fn get_mut<'a, T: Resource>(
        &'_ self,
        registry: &'a mut ResourceRegistry,
    ) -> Option<&'a mut T> {
        registry.get_mut(self)
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

/// The registry of loaded resources.
#[derive(Default)]
pub struct ResourceRegistry {
    ref_counts: Arc<Mutex<ResourceRefCounter>>,
    resources: SecondaryMap<ResourceHandleId, Option<Box<dyn Resource>>>,
}

impl ResourceRegistry {
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
    pub fn get<'a, T: Any>(&'a self, handle: &ResourceHandle) -> Option<&'a T> {
        if self
            .ref_counts
            .lock()
            .unwrap()
            .ref_counts
            .contains_key(handle.handle_id)
        {
            self.resources
                .get(handle.handle_id)?
                .as_ref()?
                .as_any()
                .downcast_ref::<T>()
        } else {
            None
        }
    }

    /// Returns a mutable reference to a resource behind the handle, None if the resource does not exist.
    pub fn get_mut<'a, T: Any>(&'a mut self, handle: &ResourceHandle) -> Option<&'a mut T> {
        if self
            .ref_counts
            .lock()
            .unwrap()
            .ref_counts
            .contains_key(handle.handle_id)
        {
            self.resources
                .get_mut(handle.handle_id)?
                .as_mut()?
                .as_any_mut()
                .downcast_mut::<T>()
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
    use std::sync::{Arc, Mutex};

    use super::{Resource, ResourceHandle, ResourceRefCounter, ResourceRegistry};

    #[test]
    fn ref_count() {
        let counter = Arc::new(Mutex::new(ResourceRefCounter::default()));

        let ref_a = ResourceHandle::new(counter.clone());
        let ref_b = ResourceHandle::new(counter);

        assert_ne!(ref_a, ref_b);
    }

    struct SampleResource;
    impl Resource for SampleResource {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    #[test]
    fn resources() {
        let mut resources = ResourceRegistry::default();

        {
            let handle = resources.insert(Box::new(SampleResource {}));
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
            let handle = resources.insert(Box::new(SampleResource {}));
            assert!(handle.get::<SampleResource>(&resources).is_some());
            assert_eq!(resources.len(), 1);
        }
    }
}
