use std::collections::hash_map::HashMap;
use std::sync::Arc;

use lgn_data_offline::resource::Project;
use lgn_data_runtime::{AssetRegistry, EditHandleUntyped, ResourceTypeAndId};
use tokio::sync::MutexGuard;

use crate::{BuildManager, Error, SelectionManager, TransactionManager};

/// Describe a Lock on the Database (Project/AssetRegistry/LoadedResources)
pub struct LockContext<'a> {
    /// Lock on the `Project`
    pub project: MutexGuard<'a, Project>,
    /// Lock on the LoadedResources
    /// Reference to the Asset Registry
    pub asset_registry: Arc<AssetRegistry>,
    /// Reference to build manager.
    pub build: MutexGuard<'a, BuildManager>,
    /// Reference to SelectionManager
    pub selection_manager: Arc<SelectionManager>,
    // List of Resource changed during the lock (that need saving)
    pub(crate) edited_resources: HashMap<ResourceTypeAndId, EditHandleUntyped>,
}

impl<'a> LockContext<'a> {
    /// Create a new Lock on the `DataManager`
    pub async fn new(transaction_manager: &'a TransactionManager) -> LockContext<'a> {
        Self {
            project: transaction_manager.project.lock().await,
            asset_registry: transaction_manager.asset_registry.clone(),
            build: transaction_manager.build_manager.lock().await,
            selection_manager: transaction_manager.selection_manager.clone(),
            edited_resources: HashMap::new(),
        }
    }

    /// Get an Handle to a Resource, load it if not in memory yet
    pub async fn edit(
        &mut self,
        resource_id: ResourceTypeAndId,
    ) -> Result<&mut EditHandleUntyped, Error> {
        #[allow(clippy::map_entry)]
        if !self.edited_resources.contains_key(&resource_id) {
            let handle = self.asset_registry.load_async_untyped(resource_id).await?;
            let edit = self.asset_registry.edit_untyped(&handle).unwrap();
            self.edited_resources.insert(resource_id, edit);
        }

        if let Some(handle) = self.edited_resources.get_mut(&resource_id) {
            return Ok(handle);
        }
        Err(Error::InvalidResource(resource_id))
    }

    pub(crate) async fn save_changed_resources(
        &mut self,
    ) -> Result<Option<Vec<ResourceTypeAndId>>, Error> {
        let mut changed: Option<Vec<ResourceTypeAndId>> = None;
        for (id, edit_handle) in self.edited_resources.drain() {
            let handle = self.asset_registry.commit_untyped(edit_handle);
            self.project
                .save_resource(handle, &self.asset_registry)
                .await
                .map_err(|err| Error::Project(id, err))?;

            changed.get_or_insert(Vec::new()).push(id);
        }

        /*for resource_id in &self.changed_resources {
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
        }*/

        Ok(changed)
    }
}
