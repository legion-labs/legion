//! Transaction Operation to Create a Resource

use async_trait::async_trait;
use lgn_data_offline::resource::ResourcePathName;
use lgn_data_runtime::ResourceTypeAndId;

use crate::{Error, LockContext, TransactionOperation};

/// Operation to Create a new Resource
pub struct CreateResourceOperation {
    resource_type_name: &'static str,
    resource_id: ResourceTypeAndId,
    resource_path: ResourcePathName,
}

impl CreateResourceOperation {
    /// Create a new `CreateResourceOperation`
    pub fn new(
        resource_type_name: &'static str,
        resource_id: ResourceTypeAndId,
        resource_path: ResourcePathName,
    ) -> Box<Self> {
        Box::new(Self {
            resource_type_name,
            resource_id,
            resource_path,
        })
    }
}

#[async_trait]
impl TransactionOperation for CreateResourceOperation {
    async fn apply_operation(&mut self, ctx: &mut LockContext<'_>) -> anyhow::Result<()> {
        let handle = ctx
            .resource_registry
            .new_resource(self.resource_id.t)
            .ok_or(Error::ResourceCreationFailed(self.resource_id.t))?;

        // Validate duplicate id/name
        if ctx.project.exists(self.resource_id) {
            return Err(Error::ResourceIdAlreadyExist(self.resource_id).into());
        }
        if ctx.project.exists_named(&self.resource_path) {
            return Err(Error::ResourcePathAlreadyExist(self.resource_path.clone()).into());
        }

        ctx.project.add_resource_with_id(
            self.resource_path.clone(),
            self.resource_type_name,
            self.resource_id.t,
            self.resource_id,
            &handle,
            &mut ctx.resource_registry,
        )?;
        ctx.loaded_resource_handles.insert(self.resource_id, handle);
        Ok(())
    }

    async fn rollback_operation(&self, ctx: &mut LockContext<'_>) -> anyhow::Result<()> {
        if let Some(_handle) = ctx.loaded_resource_handles.remove(self.resource_id) {
            ctx.project.delete_resource(self.resource_id)?;
        }
        Ok(())
    }
}
