//! Transaction Operation to Rename a Resource

use async_trait::async_trait;
use lgn_data_offline::resource::ResourcePathName;
use lgn_data_runtime::ResourceTypeAndId;

use crate::{Error, LockContext, TransactionOperation};

/// Operation to rename a Resource
#[derive(Debug)]
pub struct ReparentResourceOperation {
    resource_id: ResourceTypeAndId,
    new_path: ResourcePathName,
    old_path: Option<ResourcePathName>,
}

impl ReparentResourceOperation {
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
impl TransactionOperation for ReparentResourceOperation {
    async fn apply_operation(&mut self, ctx: &mut LockContext<'_>) -> Result<(), Error> {
        // Extract the raw name and check if it's a relative name (with the /!(PARENT_GUID)/
        let raw_name = ctx
            .project
            .raw_resource_name(self.resource_id.id)
            .map_err(|err| Error::Project(self.resource_id, err))?;

        // Check if the parent is a resource or just a path
        let parent_id = ctx.project.find_resource(&self.new_path).await.ok();
        let (_old_path, name) = raw_name
            .as_str()
            .rsplit_once("/")
            .ok_or_else(|| Error::ResourceNameNotFound(raw_name.clone()))?;

        let mut raw_path: ResourcePathName = if parent_id.is_some() {
            format!("/!{}/{}", parent_id.unwrap(), name).into()
        } else {
            format!("{}/{}", self.new_path, name).into()
        };

        raw_path = ctx.project.get_incremental_name(&raw_path).await;
        self.old_path = Some(
            ctx.project
                .rename_resource(self.resource_id, &raw_path)
                .await
                .map_err(|err| Error::Project(self.resource_id, err))?,
        );
        ctx.changed_resources.insert(self.resource_id);
        Ok(())
    }

    async fn rollback_operation(&self, ctx: &mut LockContext<'_>) -> Result<(), Error> {
        if let Some(old_path) = &self.old_path {
            ctx.project
                .rename_resource(self.resource_id, old_path)
                .await
                .map_err(|err| Error::Project(self.resource_id, err))?;
            ctx.changed_resources.insert(self.resource_id);
        }
        Ok(())
    }
}
