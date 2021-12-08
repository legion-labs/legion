use async_trait::async_trait;
use lgn_data_offline::resource::ResourcePathName;
use lgn_data_runtime::ResourceTypeAndId;

use crate::{Error, LockContext, TransactionOperation};

pub(crate) struct DeleteResourceOperation {
    resource_id: ResourceTypeAndId,
    old_resource_name: Option<ResourcePathName>,
    old_resource_data: Option<Vec<u8>>,
}

impl DeleteResourceOperation {
    pub fn new(resource_id: ResourceTypeAndId) -> Self {
        Self {
            resource_id,
            old_resource_name: None,
            old_resource_data: None,
        }
    }
}

#[async_trait]
impl TransactionOperation for DeleteResourceOperation {
    async fn apply_operation(&mut self, ctx: &mut LockContext<'_>) -> anyhow::Result<()> {
        if let Some(old_handle) = ctx.loaded_resource_handles.remove(self.resource_id) {
            // On the first apply, save a copy original resource for redo
            if self.old_resource_name.is_none() {
                let mut old_resource_data = Vec::<u8>::new();
                ctx.resource_registry.serialize_resource(
                    self.resource_id.t,
                    &old_handle,
                    &mut old_resource_data,
                )?;

                self.old_resource_name = Some(ctx.project.resource_name(self.resource_id)?);
                self.old_resource_data = Some(old_resource_data);
            }
            ctx.project.delete_resource(self.resource_id)?;
        }
        Ok(())
    }

    async fn rollback_operation(&self, ctx: &mut LockContext<'_>) -> anyhow::Result<()> {
        // Restore  resource from saved state, original name and id
        let old_resource_name = self
            .old_resource_name
            .as_ref()
            .ok_or(Error::InvalidDeleteOperation(self.resource_id))?;
        let old_resource_data = self
            .old_resource_data
            .as_ref()
            .ok_or(Error::InvalidDeleteOperation(self.resource_id))?;

        let handle = ctx
            .resource_registry
            .deserialize_resource(self.resource_id.t, &mut old_resource_data.as_slice())?;

        ctx.project.add_resource_with_id(
            old_resource_name.clone(),
            self.resource_id.t,
            self.resource_id,
            &handle,
            &mut ctx.resource_registry,
        )?;
        ctx.loaded_resource_handles.insert(self.resource_id, handle);
        Ok(())
    }
}
