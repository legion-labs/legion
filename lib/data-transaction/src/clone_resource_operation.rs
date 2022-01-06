//! Transaction Operation to Clone a Resource

use async_trait::async_trait;
use lgn_data_offline::resource::ResourcePathName;
use lgn_data_runtime::ResourceTypeAndId;

use crate::{Error, LockContext, TransactionOperation};

/// Clone a Resource Operation
pub struct CloneResourceOperation {
    source_resource_type_name: &'static str,
    source_resource_id: ResourceTypeAndId,
    clone_resource_id: ResourceTypeAndId,
    clone_path: ResourcePathName,
}

impl CloneResourceOperation {
    /// Create a new Clone a Resource Operation
    pub fn new(
        source_resource_type_name: &'static str,
        source_resource_id: ResourceTypeAndId,
        clone_resource_id: ResourceTypeAndId,
        clone_path: ResourcePathName,
    ) -> Box<Self> {
        Box::new(Self {
            source_resource_type_name,
            source_resource_id,
            clone_resource_id,
            clone_path,
        })
    }
}

#[async_trait]
impl TransactionOperation for CloneResourceOperation {
    async fn apply_operation(&mut self, ctx: &mut LockContext<'_>) -> anyhow::Result<()> {
        let source_handle = ctx
            .loaded_resource_handles
            .get(self.source_resource_id)
            .ok_or(Error::InvalidTypeReflection(self.source_resource_id))?;

        let mut buffer = Vec::<u8>::new();
        ctx.resource_registry.serialize_resource(
            self.source_resource_id.kind,
            source_handle,
            &mut buffer,
        )?;

        let clone_handle = ctx
            .resource_registry
            .deserialize_resource(self.source_resource_id.kind, &mut buffer.as_slice())?;

        ctx.project.add_resource_with_id(
            self.clone_path.clone(),
            self.source_resource_type_name,
            self.clone_resource_id.kind,
            self.clone_resource_id,
            &clone_handle,
            &mut ctx.resource_registry,
        )?;
        ctx.loaded_resource_handles
            .insert(self.clone_resource_id, clone_handle);

        Ok(())
    }

    async fn rollback_operation(&self, ctx: &mut LockContext<'_>) -> anyhow::Result<()> {
        if let Some(_clone_handle) = ctx.loaded_resource_handles.remove(self.clone_resource_id) {
            ctx.project.delete_resource(self.clone_resource_id)?;
        }
        Ok(())
    }
}
