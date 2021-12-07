use async_trait::async_trait;
use lgn_data_runtime::ResourceTypeAndId;

use crate::{Error, LockContext, TransactionOperation};

pub(crate) struct UpdatePropertyOperation {
    resource_id: ResourceTypeAndId,
    property_name: String,
    new_value: String,
    old_value: Option<String>,
}

impl UpdatePropertyOperation {
    pub fn new(resource_id: ResourceTypeAndId, property_name: &str, new_value: &str) -> Self {
        Self {
            resource_id,
            property_name: property_name.into(),
            new_value: new_value.into(),
            old_value: None,
        }
    }
}

#[async_trait]
impl TransactionOperation for UpdatePropertyOperation {
    async fn apply_operation(&mut self, ctx: &mut LockContext<'_>) -> anyhow::Result<()> {
        let resource_handle = ctx
            .loaded_resource_handles
            .get(self.resource_id)
            .ok_or(Error::InvalidResourceReflection(self.resource_id))?;

        let reflection = ctx
            .resource_registry
            .get_resource_reflection_mut(self.resource_id.0, resource_handle)
            .ok_or(Error::InvalidResourceReflection(self.resource_id))?;

        if self.old_value.is_none() {
            self.old_value = Some(reflection.read_property(self.property_name.as_str())?);
        }

        reflection.write_property(self.property_name.as_str(), self.new_value.as_str())?;
        ctx.changed_resources.insert(self.resource_id);
        Ok(())
    }

    async fn rollback_operation(&self, ctx: &mut LockContext<'_>) -> anyhow::Result<()> {
        if let Some(old_value) = &self.old_value {
            let handle = ctx
                .loaded_resource_handles
                .get(self.resource_id)
                .ok_or(Error::InvalidResource(self.resource_id))?;

            let reflection = ctx
                .resource_registry
                .get_resource_reflection_mut(self.resource_id.0, handle)
                .ok_or(Error::InvalidResourceReflection(self.resource_id))?;
            reflection.write_property(self.property_name.as_str(), old_value.as_str())?;
            ctx.changed_resources.insert(self.resource_id);
        }
        Ok(())
    }
}
