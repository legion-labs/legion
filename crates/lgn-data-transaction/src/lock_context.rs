use std::collections::hash_map::HashMap;
use std::sync::Arc;

use lgn_data_offline::Project;
use lgn_data_runtime::prelude::*;
use tokio::sync::MutexGuard;

use crate::{BuildManager, Error, SelectionManager, TransactionManager};

/// Describe a Lock on the Database (Project/AssetRegistry/LoadedResources)
pub struct LockContext<'a> {
    /// Lock on the `Project`
    pub project: MutexGuard<'a, Project>,
    /// Reference to build manager.
    pub build: MutexGuard<'a, BuildManager>,
    /// Reference to SelectionManager
    pub selection_manager: Arc<SelectionManager>,
    // List of Resource changed during the lock (that need saving)
    pub(crate) edited_resources: HashMap<ResourceTypeAndId, Box<dyn Resource>>,
}

impl<'a> LockContext<'a> {
    /// Create a new Lock on the `DataManager`
    pub async fn new(transaction_manager: &'a TransactionManager) -> LockContext<'a> {
        Self {
            project: transaction_manager.project.lock().await,
            build: transaction_manager.build_manager.lock().await,
            selection_manager: transaction_manager.selection_manager.clone(),
            edited_resources: HashMap::new(),
        }
    }

    /// Get an Handle to a Resource, load it if not in memory yet
    pub async fn new_resource(
        &mut self,
        resource_id: ResourceTypeAndId,
    ) -> Result<&mut dyn Resource, Error> {
        Ok(self
            .edited_resources
            .entry(resource_id)
            .or_insert_with(|| resource_id.kind.new_instance())
            .as_mut())
    }

    /// Get an Handle to a Resource, load it if not in memory yet
    pub async fn edit_resource(
        &mut self,
        resource_id: ResourceTypeAndId,
    ) -> Result<&mut dyn Resource, Error> {
        #[allow(clippy::map_entry)]
        if !self.edited_resources.contains_key(&resource_id) {
            let resource = self.project.load_resource_untyped(resource_id).await?;
            self.edited_resources.insert(resource_id, resource);
        }

        if let Some(handle) = self.edited_resources.get_mut(&resource_id) {
            return Ok(handle.as_mut());
        }
        Err(Error::InvalidResource(resource_id))
    }

    pub(crate) async fn save_changed_resources(
        &mut self,
    ) -> Result<Option<Vec<ResourceTypeAndId>>, Error> {
        let mut changed: Option<Vec<ResourceTypeAndId>> = None;
        for (id, resource) in self.edited_resources.drain() {
            self.project.save_resource(id.id, resource.as_ref()).await?;

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
