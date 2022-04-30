use std::collections::HashSet;
use std::sync::Arc;

use lgn_data_offline::resource::{Project, ResourceHandles, ResourceRegistry};
use lgn_data_runtime::{AssetRegistry, HandleUntyped, ResourceTypeAndId};
use lgn_tracing::error;
use tokio::sync::MutexGuard;

use crate::{BuildManager, Error, SelectionManager, TransactionManager};

/// Describe a Lock on the Database (Project/ResourceRegistry/LoadedResources)
pub struct LockContext<'a> {
    /// Lock on the `Project`
    pub project: MutexGuard<'a, Project>,
    /// Lock on the LoadedResources
    pub(crate) loaded_resource_handles: MutexGuard<'a, ResourceHandles>,
    /// Reference to the Asset Registry
    pub asset_registry: Arc<AssetRegistry>,
    /// Reference to build manager.
    pub build: MutexGuard<'a, BuildManager>,
    /// Reference to SelectionManager
    pub selection_manager: Arc<SelectionManager>,
    // List of Resource changed during the lock (that need saving)
    pub(crate) changed_resources: HashSet<ResourceTypeAndId>,
}

impl<'a> LockContext<'a> {
    /// Create a new Lock on the `DataManager`
    pub async fn new(transaction_manager: &'a TransactionManager) -> LockContext<'a> {
        Self {
            project: transaction_manager.project.lock().await,
            asset_registry: transaction_manager.asset_registry.clone(),
            build: transaction_manager.build_manager.lock().await,
            selection_manager: transaction_manager.selection_manager.clone(),
            loaded_resource_handles: transaction_manager.loaded_resource_handles.lock().await,
            changed_resources: HashSet::new(),
        }
    }

    /// Get an Handle to a Resource, load it if not in memory yet
    pub async fn get_or_load(
        &mut self,
        resource_id: ResourceTypeAndId,
    ) -> Result<HandleUntyped, Error> {
        Ok(self
            .loaded_resource_handles
            .entry(resource_id)
            .or_insert(
                self.project
                    .load_resource(resource_id, &self.asset_registry)
                    .map_err(|err| Error::Project(resource_id, err))?,
            )
            .clone())
    }

    /// Load or reload a Resource from its id
    pub async fn reload(&mut self, resource_id: ResourceTypeAndId) -> Result<(), Error> {
        let handle = self
            .project
            .load_resource(resource_id, &self.asset_registry)
            .map_err(|err| Error::Project(resource_id, err))?;

        self.loaded_resource_handles.insert(resource_id, handle);
        Ok(())
    }

    /// Unload a Resource from its id
    pub async fn unload(&mut self, resource_id: ResourceTypeAndId) {
        self.loaded_resource_handles.remove(resource_id);
    }

    pub(crate) async fn save_changed_resources(&mut self) -> Result<(), Error> {
        for resource_id in &self.changed_resources {
            if let Some(handle) = self.loaded_resource_handles.get(*resource_id) {
                self.project
                    .save_resource(*resource_id, handle.clone(), &self.asset_registry)
                    .await
                    .map_err(|err| Error::Project(*resource_id, err))?;

                self.asset_registry.reload(*resource_id);
            }
        }

        for resource_id in &self.changed_resources {
            match self
                .build
                .build_all_derived(*resource_id, &self.project)
                .await
            {
                Ok((runtime_path_id, _built_resources)) => {
                    self.asset_registry.reload(runtime_path_id.resource_id());
                }
                Err(e) => {
                    error!("Error building resource derivations {:?}", e);
                }
            }
        }

        Ok(())
    }
}
