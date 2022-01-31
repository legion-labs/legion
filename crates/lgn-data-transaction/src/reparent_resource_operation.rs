//! Transaction Operation to Rename a Resource

use async_trait::async_trait;
use lgn_data_offline::resource::ResourcePathName;
use lgn_data_runtime::ResourceTypeAndId;

use crate::{LockContext, TransactionOperation};

/// Operation to rename a Resource
pub struct ReparentResourceOperation {
    resource_id: ResourceTypeAndId,
    new_parent: ResourceTypeAndId,
    old_path: Option<ResourcePathName>,
}

impl ReparentResourceOperation {
    /// Return a newly created `RenameResourceOperation`
    pub fn new(resource_id: ResourceTypeAndId, new_parent: ResourceTypeAndId) -> Box<Self> {
        Box::new(Self {
            resource_id,
            new_parent,
            old_path: None,
        })
    }
}

#[async_trait]
impl TransactionOperation for ReparentResourceOperation {
    async fn apply_operation(&mut self, ctx: &mut LockContext<'_>) -> anyhow::Result<()> {
        // Extract the raw name and check if it's a relative name (with the /!(PARENT_GUID)/
        let mut raw_name = ctx.project.raw_resource_name(self.resource_id.id)?;

        raw_name.replace_parent_info(Some(self.new_parent), None);

        let raw_name = ctx.project.get_incremental_name(&raw_name).await;
        self.old_path = Some(
            ctx.project
                .rename_resource(self.resource_id, &raw_name)
                .await?,
        );
        Ok(())
    }

    async fn rollback_operation(&self, ctx: &mut LockContext<'_>) -> anyhow::Result<()> {
        if let Some(old_path) = &self.old_path {
            ctx.project
                .rename_resource(self.resource_id, old_path)
                .await?;
        }
        Ok(())
    }
}
