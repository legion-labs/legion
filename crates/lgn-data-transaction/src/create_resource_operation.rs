//! Transaction Operation to Create a Resource

use std::path::PathBuf;

use async_trait::async_trait;
use lgn_data_offline::ResourcePathName;
use lgn_data_runtime::{AssetRegistryReader, ResourceTypeAndId};

use crate::{Error, LockContext, TransactionOperation};

/// Operation to Create a new Resource
#[derive(Debug)]
pub struct CreateResourceOperation {
    resource_id: ResourceTypeAndId,
    resource_path: ResourcePathName,
    auto_increment_name: bool,
    content_path: Option<PathBuf>,
}

impl CreateResourceOperation {
    /// Create a new `CreateResourceOperation`
    pub fn new(
        resource_id: ResourceTypeAndId,
        resource_path: ResourcePathName,
        auto_increment_name: bool,
        content_path: Option<PathBuf>,
    ) -> Box<Self> {
        Box::new(Self {
            resource_id,
            resource_path,
            auto_increment_name,
            content_path,
        })
    }
}

#[async_trait]
impl TransactionOperation for CreateResourceOperation {
    async fn apply_operation(&mut self, ctx: &mut LockContext<'_>) -> Result<(), Error> {
        // Validate duplicate id/name
        if ctx.project.exists(self.resource_id).await {
            return Err(Error::ResourceIdAlreadyExist(self.resource_id));
        }

        let mut new_instance = if let Some(ref path) = self.content_path {
            let reader = tokio::fs::File::open(path)
                .await
                .map_err(|_err| Error::InvalidFilePath(path.clone()))?;

            let mut reader = Box::pin(reader) as AssetRegistryReader;
            // TODO: Implement proper Import
            lgn_data_offline::from_json_reader_untyped(&mut reader).await?
        } else {
            let mut new_instance = self.resource_id.kind.new_instance();
            let mut meta = lgn_data_offline::get_meta_mut(new_instance.as_mut());
            meta.type_id = self.resource_id;
            new_instance
        };

        let mut requested_resource_path = self.resource_path.clone();
        if ctx.project.exists_named(&requested_resource_path).await {
            if !self.auto_increment_name {
                return Err(Error::ResourcePathAlreadyExist(self.resource_path.clone()));
            }
            requested_resource_path = ctx
                .project
                .get_incremental_name(&requested_resource_path)
                .await;
        }

        lgn_data_offline::get_meta_mut(new_instance.as_mut()).name = requested_resource_path;

        Ok(ctx
            .project
            .add_resource_with_id(self.resource_id, new_instance.as_mut())
            .await?)
    }

    async fn rollback_operation(&self, ctx: &mut LockContext<'_>) -> Result<(), Error> {
        Ok(ctx.project.delete_resource(self.resource_id).await?)
    }
}
