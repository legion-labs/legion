//! Transaction Operation to Clone a Resource

use async_trait::async_trait;
use lgn_data_runtime::ResourceTypeAndId;

use crate::{Error, LockContext, TransactionOperation};

/// Clone a Resource Operation
pub struct CloneResourceOperation {
    source_resource_id: ResourceTypeAndId,
    clone_resource_id: ResourceTypeAndId,
    target_parent_id: Option<ResourceTypeAndId>,
}

impl CloneResourceOperation {
    /// Create a new Clone a Resource Operation
    pub fn new(
        source_resource_id: ResourceTypeAndId,
        clone_resource_id: ResourceTypeAndId,
        target_parent_id: Option<ResourceTypeAndId>,
    ) -> Box<Self> {
        Box::new(Self {
            source_resource_id,
            clone_resource_id,
            target_parent_id,
        })
    }
}

#[async_trait]
impl TransactionOperation for CloneResourceOperation {
    async fn apply_operation(&mut self, ctx: &mut LockContext<'_>) -> anyhow::Result<()> {
        let source_handle = ctx
            .loaded_resource_handles
            .get(self.source_resource_id)
            .ok_or(Error::InvalidResource(self.source_resource_id))?;

        let mut buffer = Vec::<u8>::new();
        ctx.resource_registry.serialize_resource(
            self.source_resource_id.kind,
            source_handle,
            &mut buffer,
        )?;

        let clone_handle = ctx
            .resource_registry
            .deserialize_resource(self.source_resource_id.kind, &mut buffer.as_slice())?;

        let resource_type_name = ctx
            .resource_registry
            .get_resource_type_name(self.source_resource_id.kind)
            .ok_or(Error::InvalidResourceType(self.source_resource_id.kind))?;

        // Extract the raw name and check if it's a relative name (with the /!(PARENT_GUID)/
        let mut source_raw_name = ctx.project.raw_resource_name(self.source_resource_id.id)?;
        source_raw_name.replace_parent_info(self.target_parent_id, None);

        source_raw_name = ctx.project.get_incremental_name(&source_raw_name);

        ctx.project
            .add_resource_with_id(
                source_raw_name,
                resource_type_name,
                self.clone_resource_id.kind,
                self.clone_resource_id,
                &clone_handle,
                &mut ctx.resource_registry,
            )
            .await?;

        ctx.loaded_resource_handles
            .insert(self.clone_resource_id, clone_handle);
        Ok(())
    }

    async fn rollback_operation(&self, ctx: &mut LockContext<'_>) -> anyhow::Result<()> {
        if let Some(_clone_handle) = ctx.loaded_resource_handles.remove(self.clone_resource_id) {
            ctx.project
                .delete_resource(self.clone_resource_id.id)
                .await?;
        }
        Ok(())
    }
}
