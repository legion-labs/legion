//! Transaction Operation to Rename a Resource

use async_trait::async_trait;
use lgn_data_offline::resource::ResourcePathName;
use lgn_data_runtime::ResourceTypeAndId;

use crate::{Error, LockContext, TransactionOperation};

/// Operation to rename a Resource
pub struct RenameResourceOperation {
    resource_id: ResourceTypeAndId,
    new_path: ResourcePathName,
    old_path: Option<ResourcePathName>,
}

impl RenameResourceOperation {
    /// Return a newly created `RenameResourceOperation`
    pub fn new(resource_id: ResourceTypeAndId, new_path: ResourcePathName) -> Box<Self> {
        Box::new(Self {
            resource_id,
            new_path,
            old_path: None,
        })
    }
}

#[async_trait]
impl TransactionOperation for RenameResourceOperation {
    async fn apply_operation(&mut self, ctx: &mut LockContext<'_>) -> anyhow::Result<()> {
        if !ctx.project.exists(self.resource_id.id).await {
            return Err(Error::InvalidResource(self.resource_id).into());
        }

        // Extract the raw name and check if it's a relative name (with the /!(PARENT_GUID)/
        let mut raw_name = ctx.project.raw_resource_name(self.resource_id.id)?;
        raw_name.replace_parent_info(None, Some(self.new_path.clone()));

        if ctx.project.exists_named(&raw_name) {
            return Err(Error::ResourcePathAlreadyExist(raw_name).into());
        }
        self.old_path = Some(ctx.project.rename_resource(self.resource_id, &raw_name)?);
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
