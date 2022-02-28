//! Transaction Operation to Create a Resource

use std::{fs::File, path::PathBuf};

use async_trait::async_trait;
use lgn_data_offline::resource::ResourcePathName;
use lgn_data_runtime::ResourceTypeAndId;

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
        let handle = if let Some(ref path) = self.content_path {
            let mut reader =
                File::open(path).map_err(|_err| Error::InvalidFilePath(path.clone()))?;

            ctx.resource_registry
                .deserialize_resource(self.resource_id.kind, &mut reader)
                .map_err(|err| Error::InvalidResourceDeserialization(self.resource_id, err))?
        } else {
            ctx.resource_registry
                .new_resource(self.resource_id.kind)
                .ok_or(Error::InvalidResourceType(self.resource_id.kind))?
        };

        // Validate duplicate id/name
        if ctx.project.exists(self.resource_id.id).await {
            return Err(Error::ResourceIdAlreadyExist(self.resource_id));
        }

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

        if let Some(resource_type_name) = ctx
            .resource_registry
            .get_resource_type_name(self.resource_id.kind)
        {
            ctx.project
                .add_resource_with_id(
                    requested_resource_path,
                    resource_type_name,
                    self.resource_id.kind,
                    self.resource_id.id,
                    &handle,
                    &mut ctx.resource_registry,
                )
                .await
                .map_err(|err| Error::Project(self.resource_id, err))?;

            ctx.loaded_resource_handles.insert(self.resource_id, handle);
        }
        Ok(())
    }

    async fn rollback_operation(&self, ctx: &mut LockContext<'_>) -> Result<(), Error> {
        if let Some(_handle) = ctx.loaded_resource_handles.remove(self.resource_id) {
            ctx.project
                .delete_resource(self.resource_id.id)
                .await
                .map_err(|err| Error::Project(self.resource_id, err))?;
        }
        Ok(())
    }
}
