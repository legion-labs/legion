use crate::DataManager;
use legion_data_offline::resource::{Project, ResourceHandles, ResourceRegistry};
use legion_data_offline::ResourcePathId;
use legion_data_runtime::{AssetRegistry, ResourceId, ResourceType};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::MutexGuard;

/// Describe a Lock on the Database (Project/ResourceRegistry/LoadedResources)
pub struct LockContext<'a> {
    /// Lock on the `Project`
    pub project: MutexGuard<'a, Project>,
    /// Lock on the `ResourceRegistry`
    pub resource_registry: MutexGuard<'a, ResourceRegistry>,
    /// Lock on the LoadedResources
    pub loaded_resource_handles: MutexGuard<'a, ResourceHandles>,
    /// Reference to the Asset Registry
    pub asset_registry: Arc<AssetRegistry>,
    // List of Resouce changed during the lock (that need saving)
    pub(crate) changed_resources: HashSet<ResourceId>,
}

impl<'a> LockContext<'a> {
    /// Create a new Lock on the `DataManager`
    pub async fn new(data_manager: &'a DataManager) -> LockContext<'a> {
        Self {
            project: data_manager.project.lock().await,
            resource_registry: data_manager.resource_registry.lock().await,
            asset_registry: data_manager.asset_registry.clone(),
            loaded_resource_handles: data_manager.loaded_resource_handles.lock().await,
            changed_resources: HashSet::new(),
        }
    }

    pub(crate) async fn save_changed_resources(&mut self) -> anyhow::Result<()> {
        self.changed_resources
            .iter()
            .try_for_each(|resource_id| -> anyhow::Result<()> {
                if let Some(handle) = self.loaded_resource_handles.get(*resource_id) {
                    self.project.save_resource(
                        *resource_id,
                        &handle,
                        &mut self.resource_registry,
                    )?;

                    let derived_id = ResourcePathId::from(*resource_id)
                        .push(ResourceType::new(b"runtime_debugcube"))
                        .resource_id();

                    self.asset_registry.reload(derived_id);
                }
                Ok(())
            })?;

        self.resource_registry.collect_garbage();
        Ok(())
    }
}
