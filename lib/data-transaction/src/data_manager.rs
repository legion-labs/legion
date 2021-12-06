use std::sync::Arc;

use lgn_data_offline::resource::{Project, ResourceHandles, ResourcePathName, ResourceRegistry};
use lgn_data_runtime::{AssetRegistry, ResourceId, ResourceType};
use log::info;
use thiserror::Error;
use tokio::sync::Mutex;

use crate::{LockContext, Transaction};

/// Error returned by the Transaction System.
#[derive(Error, Debug)]
pub enum Error {
    /// No active transaction
    #[error("No commit transaction available")]
    NoCommittedTransaction,

    ///Resource failed to deserializer from memory
    #[error("ResourceId '{0}' failed to deserialize")]
    InvalidResourceDeserialization(ResourceId),

    /// Resource Id Already Exists
    #[error("Resource '{0}' already exists in the Project")]
    ResourceIdAlreadyExist(ResourceId),

    /// Resource Path Already Exists
    #[error("Resource Path '{0}' already exists in the Project")]
    ResourcePathAlreadyExist(ResourcePathName),

    /// Invalid Delete Operation
    #[error("Invalid DeleteOperation on Resoruce'{0}'")]
    InvalidDeleteOperation(ResourceId),

    /// Invalid Resource
    #[error("ResourceId '{0}' not found")]
    InvalidResource(ResourceId),

    /// Resource of type failed to create
    #[error("Cannot create Resource of type {0}")]
    ResourceCreationFailed(ResourceType),

    /// Invalid Resource Reflection
    #[error("Resource {0} doesn't have reflection.")]
    InvalidResourceReflection(ResourceId),
}

/// System that manage the current state of the Loaded Offline Data
pub struct DataManager {
    commited_transactions: Vec<Transaction>,
    rollbacked_transactions: Vec<Transaction>,

    pub(crate) project: Arc<Mutex<Project>>,
    pub(crate) resource_registry: Arc<Mutex<ResourceRegistry>>,
    pub(crate) asset_registry: Arc<AssetRegistry>,
    pub(crate) loaded_resource_handles: Arc<Mutex<ResourceHandles>>,
}

impl DataManager {
    /// Create a `DataManager` from a `Project` and `ResourceRegistry`
    pub fn new(
        project: Arc<Mutex<Project>>,
        resource_registry: Arc<Mutex<ResourceRegistry>>,
        asset_registry: Arc<AssetRegistry>,
    ) -> Self {
        Self {
            commited_transactions: Vec::new(),
            rollbacked_transactions: Vec::new(),
            project,
            resource_registry,
            asset_registry,
            loaded_resource_handles: Arc::new(Mutex::new(ResourceHandles::default())),
        }
    }

    /// Load all resources from a `Project`
    pub async fn load_all_resources(&mut self) {
        let project = self.project.lock().await;
        let mut resource_registry = self.resource_registry.lock().await;
        let mut resource_handles = self.loaded_resource_handles.lock().await;

        for resource_id in project.resource_list() {
            project
                .load_resource(resource_id, &mut resource_registry)
                .map_or_else(
                    |err| log::warn!("Failed to load {}: {}", resource_id, err),
                    |handle| resource_handles.insert(resource_id, handle),
                );
        }
        info!(
            "Loaded all Project resources: {} resources loaded",
            resource_handles.resource_count()
        );
    }

    /// Commit the current pending `Transaction`
    pub async fn commit_transaction(&mut self, mut transaction: Transaction) -> anyhow::Result<()> {
        transaction
            .apply_transaction(LockContext::new(self).await)
            .await?;
        self.commited_transactions.push(transaction);
        self.rollbacked_transactions.clear();
        Ok(())
    }

    /// Undo the last commited transaction
    pub async fn undo_transaction(&mut self) -> anyhow::Result<()> {
        if let Some(mut transaction) = self.commited_transactions.pop() {
            transaction
                .rollback_transaction(LockContext::new(self).await)
                .await?;
            self.rollbacked_transactions.push(transaction);
        }
        Ok(())
    }

    /// Reapply a rollbacked transaction
    pub async fn redo_transaction(&mut self) -> anyhow::Result<()> {
        if let Some(mut transaction) = self.rollbacked_transactions.pop() {
            transaction
                .apply_transaction(LockContext::new(self).await)
                .await?;
            self.commited_transactions.push(transaction);
        }
        Ok(())
    }
}
