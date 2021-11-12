use crate::create_resource_operation::CreateResourceOperation;
use crate::delete_resource_operation::DeleteResourceOperation;
use crate::update_property_operation::UpdatePropertyOperation;

use crate::LockContext;
use legion_data_offline::resource::ResourcePathName;
use legion_data_runtime::{ResourceId, ResourceType};

use log::info;

/// Definition of a Transaction
pub struct Transaction {
    /// Transaction Unique Identifier
    id: uuid::Uuid,
    /// List of operation within the transaction
    operations: Vec<Box<dyn TransactionOperation + Send + Sync>>,
}

pub(crate) trait TransactionOperation {
    fn apply_operation(&mut self, ctx: &mut LockContext<'_>) -> anyhow::Result<()>;
    fn rollback_operation(&self, ctx: &mut LockContext<'_>) -> anyhow::Result<()>;
}

impl Transaction {
    /// Create a new Transaction
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let new_transaction = Self {
            id: uuid::Uuid::new_v4(),
            operations: Vec::new(),
        };
        info!("Begin Transaction: {}", &new_transaction.id);
        new_transaction
    }

    pub(crate) fn apply_transaction(&mut self, mut context: LockContext<'_>) -> anyhow::Result<()> {
        self.operations
            .iter_mut()
            .try_for_each(|op| op.apply_operation(&mut context))?;

        context.save_changed_resources()?;

        info!(
            "Transaction Applied: {} / {}ops",
            &self.id,
            self.operations.len()
        );
        Ok(())
    }

    pub(crate) fn roll_transaction(&self, mut context: LockContext<'_>) -> anyhow::Result<()> {
        self.operations
            .iter()
            .rev()
            .try_for_each(|op| op.rollback_operation(&mut context))?;

        context.save_changed_resources()?;

        info!(
            "Transaction Rollbacked: {} / {}ops",
            &self.id,
            self.operations.len()
        );
        Ok(())
    }

    /// Queue the Creation of a new Resource, return its `ResourceId`
    pub fn create_resource(
        &mut self,
        resource_path: ResourcePathName,
        resource_type: ResourceType,
    ) -> anyhow::Result<ResourceId> {
        let resource_id = ResourceId::new_random_id(resource_type);
        self.operations.push(Box::new(CreateResourceOperation::new(
            resource_id,
            resource_path,
        )));
        Ok(resource_id)
    }

    /// Queue the Delete of the Resources
    pub fn delete_resource(&mut self, resource_id: ResourceId) -> anyhow::Result<()> {
        self.operations
            .push(Box::new(DeleteResourceOperation::new(resource_id)));
        Ok(())
    }

    /// Queue Update of the Property of a Resource using Reflection
    pub fn update_property(
        &mut self,
        resource_id: ResourceId,
        property_name: &str,
        new_value: &str,
    ) -> anyhow::Result<()> {
        self.operations.push(Box::new(UpdatePropertyOperation::new(
            resource_id,
            property_name,
            new_value,
        )));
        Ok(())
    }
}
