use crate::{Error, LockContext, TransactionOperation};
use async_trait::async_trait;
use legion_data_offline::resource::ResourcePathName;
use legion_data_runtime::{ResourceId, ResourceType};

pub(crate) struct CreateResourceOperation {
    resource_id: (ResourceType, ResourceId),
    resource_path: ResourcePathName,
}

impl CreateResourceOperation {
    pub fn new(resource_id: (ResourceType, ResourceId), resource_path: ResourcePathName) -> Self {
        Self {
            resource_id,
            resource_path,
        }
    }
}

#[async_trait]
impl TransactionOperation for CreateResourceOperation {
    async fn apply_operation(&mut self, ctx: &mut LockContext<'_>) -> anyhow::Result<()> {
        let handle = ctx
            .resource_registry
            .new_resource(self.resource_id.0)
            .ok_or_else(|| Error::ResourceCreationFailed(self.resource_id.0))?;

        // Validate duplicate id/name
        if ctx.project.exists(self.resource_id) {
            return Err(Error::ResourceIdAlreadyExist(self.resource_id).into());
        }
        if ctx.project.exists_named(&self.resource_path) {
            return Err(Error::ResourcePathAlreadyExist(self.resource_path.clone()).into());
        }

        ctx.project.add_resource_with_id(
            self.resource_path.clone(),
            self.resource_id.0,
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
