use std::{collections::HashSet, path::PathBuf, sync::Arc};

use lgn_content_store::indexing::SharedTreeIdentifier;
use lgn_data_offline::{Project, ResourcePathName};
use lgn_data_runtime::{
    AssetRegistryError, AssetRegistryMessage, ResourcePathId, ResourceType, ResourceTypeAndId,
};
use thiserror::Error;
use tokio::sync::Mutex;

use crate::{
    build_manager::BuildManager, selection_manager::SelectionManager, LockContext, Transaction,
};

/// Error returned by the Transaction System.
#[derive(Error, Debug)]
pub enum Error {
    /// No active transaction
    #[error("No commit transaction available")]
    NoCommittedTransaction,

    /// Resource failed to deserializer from memory
    #[error("ResourceId '{0:?}' failed to deserialize")]
    InvalidResourceDeserialization(ResourceTypeAndId, AssetRegistryError),

    /// Resource failed to deserializer from memory
    #[error("ResourceId '{0:?}' failed to serialize")]
    InvalidResourceSerialization(ResourceTypeAndId, AssetRegistryError),

    /// Resource Id Already Exists
    #[error("Resource '{0:?}' already exists in the Project")]
    ResourceIdAlreadyExist(ResourceTypeAndId),

    /// Resource Path Already Exists
    #[error("Resource Path '{0}' already exists in the Project")]
    ResourcePathAlreadyExist(ResourcePathName),

    /// Resource Path Already Exists
    #[error("Resource Path '{0}' doesn't exists in the Project")]
    ResourceNameNotFound(ResourcePathName),

    /// Invalid Delete Operation
    #[error("Invalid DeleteOperation on Resource '{0:?}'")]
    InvalidDeleteOperation(ResourceTypeAndId),

    /// Invalid Resource
    #[error("ResourceId '{0:?}' not found")]
    InvalidResource(ResourceTypeAndId),

    /// Invalid Resource Reflection
    #[error("Resource '{0:?}' doesn't have reflection.")]
    InvalidTypeReflection(ResourceTypeAndId),

    /// Invalid Resource Type
    #[error("Invalid resource type '{0}'")]
    InvalidResourceType(ResourceType),

    /// Project error fallback
    #[error("Project error resource: '{0}'")]
    Project(#[from] lgn_data_offline::Error),

    /// AssetRegistry error fallback
    #[error(transparent)]
    AssetRegistry(#[from] lgn_data_runtime::AssetRegistryError),

    /// Reflection Error fallack
    #[error("Reflection error on resource '{0}': {1}")]
    Reflection(ResourceTypeAndId, lgn_data_model::ReflectionError),

    /// Reflection Error fallack
    #[error("DataBuild failed for Resource '{0:?}': {1}")]
    Databuild(ResourceTypeAndId, lgn_data_build::Error),

    /// External file loading Error
    #[error("Provided file path '{0}' couldn't be opened")]
    InvalidFilePath(PathBuf),
}

/// System that manage the current state of the Loaded Offline Data
pub struct TransactionManager {
    commited_transactions: Vec<Transaction>,
    rollbacked_transactions: Vec<Transaction>,
    notification_tx: crossbeam_channel::Sender<AssetRegistryMessage>,
    notification_rx: crossbeam_channel::Receiver<AssetRegistryMessage>,

    pub(crate) project: Arc<Mutex<Project>>,
    pub(crate) build_manager: Arc<Mutex<BuildManager>>,
    pub(crate) selection_manager: Arc<SelectionManager>,
    pub(crate) active_scenes: std::collections::HashSet<ResourceTypeAndId>,
}

impl TransactionManager {
    /// Create a `DataManager` from a `Project` and `ResourceRegistry`
    pub fn new(
        project: Arc<Mutex<Project>>,
        build_manager: BuildManager,
        selection_manager: Arc<SelectionManager>,
    ) -> Self {
        let (notification_tx, notification_rx) =
            crossbeam_channel::unbounded::<AssetRegistryMessage>();

        Self {
            commited_transactions: Vec::new(),
            rollbacked_transactions: Vec::new(),
            notification_tx,
            notification_rx,
            project,
            build_manager: Arc::new(Mutex::new(build_manager)),
            selection_manager,
            active_scenes: HashSet::new(),
        }
    }

    /// Return a Notification receiver
    pub fn get_notification_receiver(&self) -> crossbeam_channel::Receiver<AssetRegistryMessage> {
        self.notification_rx.clone()
    }

    /// Add a scene and build it
    pub async fn add_scene(
        &mut self,
        resource_id: ResourceTypeAndId,
    ) -> Result<ResourcePathId, Error> {
        self.active_scenes.insert(resource_id);
        lgn_tracing::info!("Adding scene: {}", resource_id);
        let path_id = self.build_by_id(resource_id).await?;
        let _runtime_id = path_id.resource_id();

        //if self.asset_registry.get_untyped(runtime_id).is_none() {
        //    self.asset_registry.load_untyped(runtime_id);
        //}
        Ok(path_id)
    }

    /// Remove a scene
    pub async fn remove_scene(&mut self, resource_id: ResourceTypeAndId) {
        self.active_scenes.remove(&resource_id);
        lgn_tracing::info!("Removing scene: {}", resource_id);
        //TODO: additional clean, unload from registry?
    }

    /// Get the list of active scene
    pub fn get_active_scenes(&self) -> Vec<ResourceTypeAndId> {
        self.active_scenes.iter().copied().collect()
    }

    fn notify_changed_resources(&self, changed: Option<Vec<ResourceTypeAndId>>) {
        if let Some(changed) = changed {
            let runtime_changed = changed
                .iter()
                .map(|r| BuildManager::get_derived_id(*r).resource_id())
                .collect::<Vec<_>>();

            if let Err(err) = self
                .notification_tx
                .send(AssetRegistryMessage::ChangedResources(runtime_changed))
            {
                lgn_tracing::warn!("Failed to TransactionMessage::ChangedResources: {}", err);
            }
        }
    }

    /// Build a resource by id
    pub async fn build_by_id(
        &self,
        resource_id: ResourceTypeAndId,
    ) -> Result<ResourcePathId, Error> {
        let mut ctx = LockContext::new(self).await;

        let (runtime_path_id, _changed_assets) = ctx
            .build
            .build_all_derived(resource_id, &ctx.project)
            .await
            .map_err(|err| Error::Databuild(resource_id, err))?;

        Ok(runtime_path_id)
    }

    /// Commit the current pending `Transaction`
    pub async fn commit_transaction(&mut self, mut transaction: Transaction) -> Result<(), Error> {
        let changed = transaction
            .apply_transaction(LockContext::new(self).await)
            .await?;
        self.commited_transactions.push(transaction);
        self.rollbacked_transactions.clear();
        self.notify_changed_resources(changed);
        Ok(())
    }

    /// Undo the last committed transaction
    pub async fn undo_transaction(&mut self) -> Result<(), Error> {
        if let Some(mut transaction) = self.commited_transactions.pop() {
            let changed = transaction
                .rollback_transaction(LockContext::new(self).await)
                .await?;
            self.rollbacked_transactions.push(transaction);
            self.notify_changed_resources(changed);
        }
        Ok(())
    }

    /// Reapply a rollbacked transaction
    pub async fn redo_transaction(&mut self) -> Result<(), Error> {
        if let Some(mut transaction) = self.rollbacked_transactions.pop() {
            let changed = transaction
                .apply_transaction(LockContext::new(self).await)
                .await?;
            self.commited_transactions.push(transaction);
            self.notify_changed_resources(changed);
        }
        Ok(())
    }

    /// Retrieve the identifier for the current runtime manifest
    pub async fn get_runtime_manifest_id(&self) -> SharedTreeIdentifier {
        self.build_manager.lock().await.get_runtime_manifest_id()
    }
}
