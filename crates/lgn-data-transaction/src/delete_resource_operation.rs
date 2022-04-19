//! Transaction Operation to Delete a Resource

use async_trait::async_trait;
use lgn_data_offline::resource::ResourcePathName;
use lgn_data_runtime::{AssetRegistryReader, ResourceTypeAndId};

use crate::{Error, LockContext, TransactionOperation};

/// Operation to Delete a resource
#[derive(Debug)]
pub struct DeleteResourceOperation {
    resource_id: ResourceTypeAndId,
    old_resource_name: Option<ResourcePathName>,
    old_resource_data: Option<Vec<u8>>,
}

impl DeleteResourceOperation {
    /// Return a newly created `DeleteResourceOperation`
    pub fn new(resource_id: ResourceTypeAndId) -> Box<Self> {
        Box::new(Self {
            resource_id,
            old_resource_name: None,
            old_resource_data: None,
        })
    }
}

#[async_trait]
impl TransactionOperation for DeleteResourceOperation {
    async fn apply_operation(&mut self, ctx: &mut LockContext<'_>) -> Result<(), Error> {
        // Force load to retrieve of value
        if let Ok(old_handle) = ctx
            .asset_registry
            .load_async_untyped(self.resource_id)
            .await
        {
            // On the first apply, save a copy original resource for redo
            if self.old_resource_name.is_none() {
                let mut old_resource_data = Vec::<u8>::new();
                ctx.asset_registry
                    .serialize_resource(old_handle, &mut old_resource_data)
                    .map_err(|err| Error::InvalidResourceSerialization(self.resource_id, err))?;

                self.old_resource_name = Some(
                    ctx.project
                        .raw_resource_name(self.resource_id.id)
                        .map_err(|err| Error::Project(self.resource_id, err))?,
                );
                self.old_resource_data = Some(old_resource_data);
            }
        }
        ctx.project
            .delete_resource(self.resource_id.id)
            .await
            .map_err(|err| Error::Project(self.resource_id, err))?;
        Ok(())
    }

    async fn rollback_operation(&self, ctx: &mut LockContext<'_>) -> Result<(), Error> {
        // Restore  resource from saved state, original name and id
        let old_resource_name = self
            .old_resource_name
            .as_ref()
            .ok_or(Error::InvalidDeleteOperation(self.resource_id))?;
        let old_resource_data = self
            .old_resource_data
            .as_ref()
            .ok_or(Error::InvalidDeleteOperation(self.resource_id))?;

        let reader =
            Box::pin(std::io::Cursor::new(old_resource_data.clone())) as AssetRegistryReader;

        let handle = ctx
            .asset_registry
            .deserialize_resource(self.resource_id, reader)
            .await
            .map_err(|err| Error::InvalidResourceDeserialization(self.resource_id, err))?;

        ctx.project
            .add_resource(old_resource_name.clone(), &handle, &ctx.asset_registry)
            .await
            .map_err(|err| Error::Project(self.resource_id, err))?;
        Ok(())
    }
}
