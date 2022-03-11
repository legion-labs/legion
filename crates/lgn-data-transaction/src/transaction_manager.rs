use std::{path::PathBuf, sync::Arc};

use lgn_data_offline::resource::{
    Project, ResourceHandles, ResourcePathName, ResourceRegistry, ResourceRegistryError,
};
use lgn_data_runtime::{AssetRegistry, ResourceType, ResourceTypeAndId};
use lgn_tracing::{info, warn};
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
    InvalidResourceDeserialization(ResourceTypeAndId, ResourceRegistryError),

    /// Resource failed to deserializer from memory
    #[error("ResourceId '{0:?}' failed to serialize")]
    InvalidResourceSerialization(ResourceTypeAndId, ResourceRegistryError),

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

    /// Project failed to flush itself
    #[error("Failed to open Project '{0}'")]
    ProjectFailedOpen(String),

    /// Project failed to flush itself
    #[error("Project flush failed: {0}")]
    ProjectFlushFailed(lgn_data_offline::resource::Error),

    /// Project error fallback
    #[error("Project error resource '{0}': {1}")]
    Project(ResourceTypeAndId, lgn_data_offline::resource::Error),

    /// Reflection Error fallack
    #[error("Reflection error on resource '{0}': {1}")]
    Reflection(ResourceTypeAndId, lgn_data_model::ReflectionError),

    /// Reflection Error fallack
    #[error("DataBuild failed fro Resource '{0:?}': {1}")]
    Databuild(ResourceTypeAndId, lgn_data_build::Error),

    /// External file loading Error
    #[error("Provided file path '{0}' couldn't be opened")]
    InvalidFilePath(PathBuf),
}

/// System that manage the current state of the Loaded Offline Data
pub struct TransactionManager {
    commited_transactions: Vec<Transaction>,
    rollbacked_transactions: Vec<Transaction>,
    pub(crate) loaded_resource_handles: Arc<Mutex<ResourceHandles>>,

    pub(crate) project: Arc<Mutex<Project>>,
    pub(crate) resource_registry: Arc<Mutex<ResourceRegistry>>,
    pub(crate) asset_registry: Arc<AssetRegistry>,
    pub(crate) build_manager: Arc<Mutex<BuildManager>>,
    pub(crate) selection_manager: Arc<SelectionManager>,
}

impl TransactionManager {
    /// Create a `DataManager` from a `Project` and `ResourceRegistry`
    pub fn new(
        project: Arc<Mutex<Project>>,
        resource_registry: Arc<Mutex<ResourceRegistry>>,
        asset_registry: Arc<AssetRegistry>,
        build_manager: BuildManager,
        selection_manager: Arc<SelectionManager>,
    ) -> Self {
        Self {
            commited_transactions: Vec::new(),
            rollbacked_transactions: Vec::new(),
            project,
            resource_registry,
            asset_registry,
            loaded_resource_handles: Arc::new(Mutex::new(ResourceHandles::default())),
            build_manager: Arc::new(Mutex::new(build_manager)),
            selection_manager,
        }
    }

    /// Build a resource by id
    pub async fn build_by_id(&self, resource_id: ResourceTypeAndId) -> Result<(), Error> {
        let mut ctx = LockContext::new(self).await;

        let (_runtime_path_id, changed_assets) = ctx
            .build
            .build_all_derived(resource_id, &ctx.project)
            .await
            .map_err(|err| Error::Databuild(resource_id, err))?;

        for asset_id in changed_assets {
            ctx.asset_registry.load_untyped(asset_id);
        }
        Ok(())
    }

    /// Build a resource by name
    pub async fn build_by_name(&self, resource_path: &ResourcePathName) -> Result<(), Error> {
        let resource_id = {
            let ctx = LockContext::new(self).await;
            ctx.project
                .find_resource(resource_path)
                .await
                .map_err(|_err| Error::ResourceNameNotFound(resource_path.clone()))?
        };
        self.build_by_id(resource_id).await
    }

    /// Load all resources from a `Project`
    pub async fn load_all_resources(&mut self) {
        let project = self.project.lock().await;
        let mut resource_registry = self.resource_registry.lock().await;
        let mut resource_handles = self.loaded_resource_handles.lock().await;

        for resource_id in project.resource_list().await {
            let kind = project.resource_type(resource_id).ok();
            if kind.is_none() {
                warn!("Skipping unknown resource type for Id:{}", resource_id);
                continue;
            }
            let kind = kind.unwrap();

            let type_id = ResourceTypeAndId {
                kind,
                id: resource_id,
            };

            let start = std::time::Instant::now();
            project
                .load_resource(type_id, &mut resource_registry)
                .map_or_else(
                    |err| {
                        warn!("Failed to load {}: {}", type_id, err);
                    },
                    |handle| resource_handles.insert(type_id, handle),
                );

            info!(
                "Loaded resource {} {} in ({:?})",
                resource_id,
                kind.as_pretty(),
                start.elapsed(),
            );
        }
        info!(
            "Loaded all Project resources: {} resources loaded",
            resource_handles.resource_count()
        );
    }

    /// Commit the current pending `Transaction`
    pub async fn commit_transaction(&mut self, mut transaction: Transaction) -> Result<(), Error> {
        transaction
            .apply_transaction(LockContext::new(self).await)
            .await?;
        self.commited_transactions.push(transaction);
        self.rollbacked_transactions.clear();
        Ok(())
    }

    /// Undo the last committed transaction
    pub async fn undo_transaction(&mut self) -> Result<(), Error> {
        if let Some(mut transaction) = self.commited_transactions.pop() {
            transaction
                .rollback_transaction(LockContext::new(self).await)
                .await?;
            self.rollbacked_transactions.push(transaction);
        }
        Ok(())
    }

    /// Reapply a rollbacked transaction
    pub async fn redo_transaction(&mut self) -> Result<(), Error> {
        if let Some(mut transaction) = self.rollbacked_transactions.pop() {
            transaction
                .apply_transaction(LockContext::new(self).await)
                .await?;
            self.commited_transactions.push(transaction);
        }
        Ok(())
    }
}
