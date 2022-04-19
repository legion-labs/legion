//! Transaction Operation to Delete a Resource

use std::fmt;

use async_trait::async_trait;
use lgn_data_runtime::{Resource, ResourceTypeAndId};

use crate::{Error, LockContext, TransactionOperation};

/// Operation to Delete a resource
//#[derive(Debug)]
pub struct DeleteResourceOperation {
    resource_id: ResourceTypeAndId,
    old_resource_data: Option<Box<dyn Resource>>,
}

impl fmt::Debug for DeleteResourceOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self.resource_id))
    }
}

impl DeleteResourceOperation {
    /// Return a newly created `DeleteResourceOperation`
    pub fn new(resource_id: ResourceTypeAndId) -> Box<Self> {
        Box::new(Self {
            resource_id,
            old_resource_data: None,
        })
    }
}

#[async_trait]
impl TransactionOperation for DeleteResourceOperation {
    async fn apply_operation(&mut self, ctx: &mut LockContext<'_>) -> Result<(), Error> {
        // Force load to retrieve of value
        if let Ok(old_resource_data) = ctx.project.load_resource_untyped(self.resource_id).await {
            self.old_resource_data = Some(old_resource_data);
        }
        Ok(ctx.project.delete_resource(self.resource_id.id).await?)
    }

    async fn rollback_operation(&self, ctx: &mut LockContext<'_>) -> Result<(), Error> {
        // Restore  resource from saved state, original name and id
        let old_resource_data = self
            .old_resource_data
            .as_ref()
            .ok_or(Error::InvalidDeleteOperation(self.resource_id))?;

        Ok(ctx
            .project
            .add_resource_with_id(self.resource_id.id, old_resource_data.as_ref())
            .await?)
    }
}
