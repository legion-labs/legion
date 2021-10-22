use crate::DataManager;
use legion_data_offline::resource::{Project, ResourceHandles, ResourceRegistry};
use legion_data_runtime::ResourceId;
use std::collections::HashSet;
use std::sync::MutexGuard;

/// Describe a Lock on the Database (Project/ResourceRegistry/LoadedResources)
pub struct LockContext<'a> {
    /// Lock on the `Project`
    pub project: MutexGuard<'a, Project>,
    /// Lock on the `ResourceRegistry`
    pub resource_registry: MutexGuard<'a, ResourceRegistry>,
    /// Lock on the LoadedResources
    pub loaded_resource_handles: MutexGuard<'a, ResourceHandles>,
    // List of Resouce changed during the lock (that need saving)
    pub(crate) changed_resources: HashSet<ResourceId>,
}

impl<'a> LockContext<'a> {
    /// Create a new Lock on the `DataManager`
    pub fn new(data_manager: &'a DataManager) -> LockContext<'a> {
        Self {
            project: data_manager.project.lock().unwrap(),
            resource_registry: data_manager.resource_registry.lock().unwrap(),
            loaded_resource_handles: data_manager.loaded_resource_handles.lock().unwrap(),
            changed_resources: HashSet::new(),
        }
    }

    pub(crate) fn save_changed_resources(&mut self) -> anyhow::Result<()> {
        self.changed_resources
            .iter()
            .try_for_each(|resource_id| -> anyhow::Result<()> {
                if let Some(handle) = self.loaded_resource_handles.get(*resource_id) {
                    self.project.save_resource(
                        *resource_id,
                        &handle,
                        &mut self.resource_registry,
                    )?;
                }
                Ok(())
            })?;

        self.resource_registry.collect_garbage();
        Ok(())
    }
}
